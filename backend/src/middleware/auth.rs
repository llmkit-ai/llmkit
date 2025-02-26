use axum::{
    body::Body,
    extract::State,
    http::Request,
    middleware::Next,
    response::Response,
};

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
