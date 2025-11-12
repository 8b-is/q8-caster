use axum::{
    Router,
    routing::{get, post},
    response::Html,
    extract::State,
    http::StatusCode,
    Json,
};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use crate::{Result, CasterError};
use crate::display::DisplayManager;
use crate::media::MediaEngine;
use crate::render::RenderEngine;
use crate::network::NetworkReceiver;
use crate::cache::ContentCache;
use crate::secrets::{SecretsManager, keycloak::{KeycloakAuth, login_handler, callback_handler, logout_handler, userinfo_handler}};

use super::api;
use super::sse::sse_handler;
use super::auth::AuthLayer;

pub struct HttpServer {
    pub display_manager: Arc<RwLock<DisplayManager>>,
    pub media_engine: Arc<RwLock<MediaEngine>>,
    pub render_engine: Arc<RwLock<RenderEngine>>,
    pub network_receiver: Arc<RwLock<NetworkReceiver>>,
    pub content_cache: Arc<RwLock<ContentCache>>,
    pub secrets_manager: Arc<SecretsManager>,
    pub keycloak_auth: Arc<KeycloakAuth>,
}

#[derive(Clone)]
pub struct AppState {
    pub display_manager: Arc<RwLock<DisplayManager>>,
    pub media_engine: Arc<RwLock<MediaEngine>>,
    pub render_engine: Arc<RwLock<RenderEngine>>,
    pub network_receiver: Arc<RwLock<NetworkReceiver>>,
    pub content_cache: Arc<RwLock<ContentCache>>,
    pub secrets_manager: Arc<SecretsManager>,
    pub keycloak_auth: Arc<KeycloakAuth>,
}

impl HttpServer {
    pub async fn new() -> Result<Self> {
        let secrets_manager = Arc::new(SecretsManager::new().await?);
        let keycloak_config = secrets_manager.get_keycloak_config().clone();
        let keycloak_auth = Arc::new(KeycloakAuth::new(keycloak_config).await?);
        
        Ok(Self {
            display_manager: Arc::new(RwLock::new(DisplayManager::new().await?)),
            media_engine: Arc::new(RwLock::new(MediaEngine::new()?)),
            render_engine: Arc::new(RwLock::new(RenderEngine::new().await?)),
            network_receiver: Arc::new(RwLock::new(NetworkReceiver::new().await?)),
            content_cache: Arc::new(RwLock::new(ContentCache::new()?)),
            secrets_manager,
            keycloak_auth,
        })
    }

    pub async fn run(self, port: u16) -> Result<()> {
        let state = AppState {
            display_manager: Arc::clone(&self.display_manager),
            media_engine: Arc::clone(&self.media_engine),
            render_engine: Arc::clone(&self.render_engine),
            network_receiver: Arc::clone(&self.network_receiver),
            content_cache: Arc::clone(&self.content_cache),
            secrets_manager: Arc::clone(&self.secrets_manager),
            keycloak_auth: Arc::clone(&self.keycloak_auth),
        };

        let app = Router::new()
            // Public routes (no auth required)
            .route("/", get(dashboard))
            .route("/health", get(health_check))
            .route("/auth/login", get(login_handler))
            .route("/auth/callback", get(callback_handler))
            .route("/auth/logout", post(logout_handler))
            .route("/auth/userinfo", get(userinfo_handler))
            
            // SSE endpoint for real-time updates
            .route("/events", get(sse_handler))
            
            // Protected API endpoints
            .route("/api/displays", get(api::list_displays))
            .route("/api/displays/:id/cast", post(api::cast_content))
            .route("/api/displays/:id/stop", post(api::stop_cast))
            .route("/api/displays/:id/configure", post(api::configure_display))
            
            .route("/api/codecs", get(api::list_codecs))
            .route("/api/audio", get(api::list_audio_devices))
            
            .route("/api/chromecast/discover", get(api::discover_chromecasts))
            .route("/api/chromecast/:name/connect", post(api::connect_chromecast))
            .route("/api/chromecast/:name/cast", post(api::cast_to_chromecast))
            .route("/api/chromecast/:name/control", post(api::control_chromecast))
            
            .route("/api/receiver/start", post(api::start_receiver))
            .route("/api/cache", post(api::cache_content))
            
            // Secrets management endpoints
            .route("/api/secrets/api-keys", post(api::add_api_key))
            .route("/api/secrets/rtsp-credentials", post(api::add_rtsp_credential))
            
            // Add state
            .with_state(state)
            
            // Add middleware
            .layer(CorsLayer::permissive())
            .layer(TraceLayer::new_for_http())
            .layer(AuthLayer::with_keycloak(Arc::clone(&self.keycloak_auth)));

        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        info!("Q8-Caster HTTP server listening on http://{}", addr);
        
        let listener = tokio::net::TcpListener::bind(addr).await
            .map_err(|e| CasterError::Network(format!("Failed to bind to port {}: {}", port, e)))?;
            
        axum::serve(listener, app).await
            .map_err(|e| CasterError::Network(format!("Server error: {}", e)))?;
        
        Ok(())
    }
}

async fn dashboard() -> Html<&'static str> {
    Html(include_str!("../../static/dashboard.html"))
}

async fn health_check() -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::OK, Json(serde_json::json!({
        "status": "healthy",
        "service": "q8-caster",
        "version": env!("CARGO_PKG_VERSION"),
    })))
}