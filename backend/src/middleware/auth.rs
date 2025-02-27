use axum::{
    body::Body,
    extract::State,
    http::{Request, HeaderMap, header::COOKIE},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{AppError, AppState};
use crate::db::types::user::{UserRole, RegistrationState};

// Constants for authentication
pub const JWT_COOKIE_NAME: &str = "llmkit-auth";
pub const JWT_EXPIRY_SECONDS: u64 = 24 * 3600; // 24 hours

// JWT Configuration
static JWT_SECRET: OnceLock<String> = OnceLock::new();

fn get_jwt_secret() -> &'static str {
    JWT_SECRET.get_or_init(|| {
        std::env::var("JWT_SECRET").unwrap_or_else(|_| "default_jwt_secret_please_change_in_production".to_string())
    })
}

// JWT Claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // Subject (user ID)
    pub role: String, // User role
    pub exp: usize,   // Expiration time
    pub iat: usize,   // Issued at
}

// Create a JWT for a user
pub fn create_jwt(user_id: i64, role: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() + JWT_EXPIRY_SECONDS;
    
    let claims = Claims {
        sub: user_id.to_string(),
        role: role.to_string(),
        exp: expiration as usize,
        iat: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(get_jwt_secret().as_bytes()),
    )
}

// Validate a JWT and extract the claims
pub fn validate_jwt(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token = decode::<Claims>(
        token,
        &DecodingKey::from_secret(get_jwt_secret().as_bytes()),
        &Validation::new(Algorithm::HS256),
    )?;
    
    Ok(token.claims)
}

// Extract claims from request (from cookie only)
pub fn extract_claims_from_request(req: &Request<Body>) -> Option<Claims> {
    extract_token_from_cookie(req.headers())
        .and_then(|token| validate_jwt(&token).ok())
}

// Extract token from cookie
pub fn extract_token_from_cookie(headers: &HeaderMap) -> Option<String> {
    headers
        .get(COOKIE)
        .and_then(|cookie_header| cookie_header.to_str().ok())
        .and_then(|cookie_str| {
            let cookie_prefix = format!("{}=", JWT_COOKIE_NAME);
            cookie_str.split(';')
                .find(|s| s.trim().starts_with(&cookie_prefix))
                .map(|jwt_cookie| {
                    let parts: Vec<&str> = jwt_cookie.split('=').collect();
                    if parts.len() == 2 {
                        parts[1].trim().to_string()
                    } else {
                        String::new()
                    }
                })
                .filter(|s| !s.is_empty())
        })
}

// Extract user ID from JWT claims
pub fn get_user_id_from_claims(claims: &Claims) -> Result<i64, AppError> {
    claims.sub.parse::<i64>().map_err(|_| 
        AppError::Unauthorized("Invalid user ID in token".to_string())
    )
}

// Middleware to verify API keys (using Authorization header)
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

// Middleware to verify user authentication using JWT from cookie only
pub async fn user_auth_middleware(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    // Extract claims from cookie
    let claims = extract_claims_from_request(&req)
        .ok_or_else(|| AppError::Unauthorized("Valid authentication cookie required".to_string()))?;
    
    // Get user ID from claims
    let user_id = get_user_id_from_claims(&claims)?;
    
    // Check if the user exists and is approved
    let user = state.db.user.get_user_by_id(user_id).await
        .map_err(|e| AppError::InternalServerError(format!("Failed to verify user: {}", e)))?
        .ok_or_else(|| AppError::Unauthorized("User not found".to_string()))?;
    
    if user.registration_state_id != RegistrationState::Approved.to_id() {
        return Err(AppError::Unauthorized("User account is not approved".to_string()));
    }
    
    // User is authenticated and approved, proceed
    Ok(next.run(req).await)
}

// Middleware to verify admin role using JWT from cookie only
pub async fn admin_auth_middleware(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    // Extract claims from cookie
    let claims = extract_claims_from_request(&req)
        .ok_or_else(|| AppError::Unauthorized("Valid authentication cookie required".to_string()))?;
    
    // Get user ID from claims
    let user_id = get_user_id_from_claims(&claims)?;
    
    // Verify the role claim
    if claims.role != "admin" {
        return Err(AppError::Unauthorized("Admin access required".to_string()));
    }
    
    // Double-check with the database (to ensure role wasn't revoked)
    let user = state.db.user.get_user_by_id(user_id).await
        .map_err(|e| AppError::InternalServerError(format!("Failed to verify user: {}", e)))?
        .ok_or_else(|| AppError::Unauthorized("User not found".to_string()))?;
    
    if user.role_id != UserRole::Admin.to_id() {
        return Err(AppError::Unauthorized("Admin access required".to_string()));
    }
    
    // User is an admin, proceed
    Ok(next.run(req).await)
}
