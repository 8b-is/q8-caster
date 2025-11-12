use axum::{
    extract::{State, Path, Json},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::json;
use tracing::info;
use uuid::Uuid;

use super::http::AppState;
use super::sse::{notify_cast_started, notify_cast_stopped, notify_error};
use crate::{ContentType, ContentSource, StreamProtocol};
use secrecy::ExposeSecret;

// Display endpoints
pub async fn list_displays(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let display_manager = state.display_manager.read().await;
    let displays = display_manager.list_displays().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(json!({
        "displays": displays
    })))
}

pub async fn cast_content(
    State(state): State<AppState>,
    Path(display_id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let content_type = payload["content_type"].as_str().unwrap_or("");
    let source = payload["source"].as_str().unwrap_or("");
    let options = &payload["options"];
    
    info!("Casting {} to display {}", content_type, display_id);
    
    // Create session
    let session_id = Uuid::new_v4().to_string();
    
    // Notify via SSE
    notify_cast_started(display_id.clone(), content_type.to_string(), session_id.clone());
    
    Ok(Json(json!({
        "success": true,
        "session_id": session_id,
        "display_id": display_id
    })))
}

pub async fn stop_cast(
    State(_state): State<AppState>,
    Path(display_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Stopping cast on display {}", display_id);
    
    // TODO: Get actual session ID
    let session_id = "mock-session";
    
    notify_cast_stopped(display_id.clone(), session_id.to_string());
    
    Ok(Json(json!({
        "success": true,
        "display_id": display_id
    })))
}

pub async fn configure_display(
    State(_state): State<AppState>,
    Path(display_id): Path<String>,
    Json(config): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Configuring display {}: {:?}", display_id, config);
    
    Ok(Json(json!({
        "success": true,
        "display_id": display_id
    })))
}

// Media endpoints
pub async fn list_codecs(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let media_engine = state.media_engine.read().await;
    let codecs = media_engine.list_codecs()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(json!({
        "codecs": codecs
    })))
}

pub async fn list_audio_devices(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let media_engine = state.media_engine.read().await;
    let devices = media_engine.list_audio_devices()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(json!({
        "audio_devices": devices
    })))
}

// Chromecast endpoints
pub async fn discover_chromecasts(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut network_receiver = state.network_receiver.write().await;
    let devices = network_receiver.discover_chromecasts().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(json!({
        "devices": devices
    })))
}

pub async fn connect_chromecast(
    State(state): State<AppState>,
    Path(device_name): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Connecting to Chromecast: {}", device_name);
    
    let mut network_receiver = state.network_receiver.write().await;
    match network_receiver.connect_chromecast(&device_name).await {
        Ok(_) => Ok(Json(json!({
            "success": true,
            "device_name": device_name
        }))),
        Err(e) => {
            notify_error(format!("Failed to connect to {}: {}", device_name, e));
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn cast_to_chromecast(
    State(state): State<AppState>,
    Path(device_name): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let content_type = payload["content_type"].as_str().unwrap_or("");
    let source = payload["source"].as_str().unwrap_or("");
    
    info!("Casting to Chromecast {} - type: {}", device_name, content_type);
    
    // TODO: Convert JSON to proper types
    
    Ok(Json(json!({
        "success": true,
        "device_name": device_name
    })))
}

pub async fn control_chromecast(
    State(state): State<AppState>,
    Path(device_name): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let action = payload["action"].as_str().unwrap_or("");
    
    info!("Controlling Chromecast {} - action: {}", device_name, action);
    
    match action {
        "stop" => {
            let mut network_receiver = state.network_receiver.write().await;
            match network_receiver.stop_chromecast(&device_name).await {
                Ok(_) => Ok(Json(json!({"success": true}))),
                Err(e) => {
                    notify_error(format!("Failed to stop {}: {}", device_name, e));
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        },
        _ => Ok(Json(json!({
            "success": false,
            "error": format!("Unknown action: {}", action)
        })))
    }
}

// Network receiver
pub async fn start_receiver(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let protocols = payload["protocols"].as_array()
        .map(|arr| arr.iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
        )
        .unwrap_or_else(|| vec!["upnp".to_string(), "airplay".to_string()]);
    
    let port = payload["port"].as_u64().unwrap_or(8420) as u16;
    
    info!("Starting receivers: {:?} on port {}", protocols, port);
    
    let mut network_receiver = state.network_receiver.write().await;
    network_receiver.start(protocols, port).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(json!({
        "success": true,
        "port": port
    })))
}

// Cache
pub async fn cache_content(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let key = payload["key"].as_str().unwrap_or("");
    let source = payload["source"].as_str().unwrap_or("");
    let ttl = payload["ttl"].as_u64().map(|t| std::time::Duration::from_secs(t));
    
    info!("Caching content: {} -> {}", source, key);
    
    let mut cache = state.content_cache.write().await;
    cache.store(key.to_string(), source.to_string(), ttl).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(json!({
        "success": true,
        "key": key
    })))
}

// Secrets management endpoints
pub async fn add_api_key(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let name = payload["name"].as_str()
        .ok_or(StatusCode::BAD_REQUEST)?;
    let key = payload["key"].as_str()
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    info!("Adding API key: {}", name);
    
    state.secrets_manager.add_api_key(name.to_string(), key.to_string()).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(json!({
        "success": true,
        "name": name
    })))
}

pub async fn add_rtsp_credential(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let camera_id = payload["camera_id"].as_str()
        .ok_or(StatusCode::BAD_REQUEST)?;
    let username = payload["username"].as_str()
        .ok_or(StatusCode::BAD_REQUEST)?;
    let password = payload["password"].as_str()
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    info!("Adding RTSP credential for camera: {}", camera_id);
    
    state.secrets_manager.add_rtsp_credential(
        camera_id.to_string(),
        username.to_string(),
        password.to_string()
    ).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(json!({
        "success": true,
        "camera_id": camera_id
    })))
}