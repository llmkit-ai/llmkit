use axum::{
    body::Body,
    extract::State,
    http::Request,
    middleware::Next,
    response::Response,
};

use serde::{Deserialize, Serialize};
use tower_cookies::Cookies;
use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::{AppError, AppState};

pub async fn api_key_middleware(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|header| header.to_str().ok());

    match auth_header {
        Some(auth) if auth.starts_with("Bearer ") => {
            let api_key = &auth[7..]; // Remove "Bearer " prefix
            
            if api_key.is_empty() {
                return Err(AppError::Unauthorized("API key is required".to_string()));
            }

            // Verify API key
            let is_valid = state
                .db
                .api_key
                .verify_any_api_key(api_key)
                .await
                .map_err(|e| {
                    AppError::InternalServerError(format!("Failed to verify API key: {}", e))
                })?;

            if !is_valid {
                return Err(AppError::Unauthorized("Invalid API key".to_string()));
            }

            // API key is valid, proceed
            Ok(next.run(req).await)
        }
        _ => Err(AppError::Unauthorized("API key is required".to_string())),
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    user_id: i64,
    exp: u64,
}

#[derive(Debug, Clone)]
pub struct UserId(pub i64);

pub async fn user_auth_middleware(
    State(state): State<AppState>,
    cookies: Cookies,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    match cookies.get("llmkit_auth_token") {
        Some(token) => {
            let token = token.value();
            let key = state.jwt_secret.as_bytes();

            match decode::<Claims>(&token, &DecodingKey::from_secret(key), &Validation::default()) {
                Ok(t) => {
                    req.extensions_mut().insert(UserId(t.claims.user_id));
                    Ok(next.run(req).await)
                },
                Err(e) => {
                    tracing::error!("Failed to decode JWT | {}", e);
                    return Err(AppError::Unauthorized("Unauthorized".to_string()))
                }
            }
        }
        _ => Err(AppError::Unauthorized("Auth token is required".to_string())),
    }
}
