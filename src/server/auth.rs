use axum::{
    extract::Request,
    http::{StatusCode, HeaderMap},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Arc;
use crate::secrets::keycloak::KeycloakAuth;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: u64,
    pub role: String,
}

pub struct AuthLayer {
    secret: String,
    keycloak: Option<Arc<KeycloakAuth>>,
}

impl AuthLayer {
    pub fn new(secret: &str) -> Self {
        Self {
            secret: secret.to_string(),
            keycloak: None,
        }
    }
    
    pub fn with_keycloak(keycloak: Arc<KeycloakAuth>) -> Self {
        Self {
            secret: String::new(),
            keycloak: Some(keycloak),
        }
    }
}

impl<S> tower::Layer<S> for AuthLayer {
    type Service = AuthMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthMiddleware {
            inner,
            secret: self.secret.clone(),
            keycloak: self.keycloak.clone(),
        }
    }
}

#[derive(Clone)]
pub struct AuthMiddleware<S> {
    inner: S,
    secret: String,
    keycloak: Option<Arc<KeycloakAuth>>,
}

impl<S> tower::Service<Request> for AuthMiddleware<S>
where
    S: tower::Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let inner = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, inner);
        let secret = self.secret.clone();
        let keycloak = self.keycloak.clone();

        Box::pin(async move {
            // Skip auth for public endpoints
            let path = request.uri().path();
            if path == "/" || 
               path == "/health" || 
               path == "/events" || 
               path.starts_with("/auth/") ||
               path.starts_with("/static/") {
                return inner.call(request).await;
            }

            // Check for authorization
            let headers = request.headers();
            let authorized = if let Some(keycloak) = keycloak {
                check_keycloak_auth(headers, &keycloak).await
            } else {
                check_auth(headers, &secret)
            };

            if !authorized {
                return Ok(Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .body(axum::body::Body::from("Unauthorized"))
                    .unwrap());
            }

            inner.call(request).await
        })
    }
}

fn check_auth(headers: &HeaderMap, _secret: &str) -> bool {
    // Simple API key check for now
    if let Some(api_key) = headers.get("x-api-key") {
        if api_key == "q8-caster-dev-key" {
            return true;
        }
    }
    
    // TODO: Implement JWT validation
    false
}

async fn check_keycloak_auth(headers: &HeaderMap, keycloak: &Arc<KeycloakAuth>) -> bool {
    if let Some(auth_header) = headers.get(axum::http::header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                if let Ok(_) = keycloak.validate_token(token).await {
                    return true;
                }
            }
        }
    }
    false
}

pub fn create_token(user_id: &str, role: &str, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() + 3600 * 24; // 24 hours

    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration,
        role: role.to_string(),
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
}