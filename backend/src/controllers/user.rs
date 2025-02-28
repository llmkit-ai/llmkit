use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher, SaltString
    },
    Argon2, PasswordHash, PasswordVerifier
};

use axum::{
    extract::State, response::IntoResponse, Json
};
use hyper::{header::SET_COOKIE, HeaderMap, StatusCode};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::env;

use crate::{AppError, AppState};
use super::types::request::user::{RegisterRequest, LoginRequest};


#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    user_id: i64,
    exp: u64,
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AppError> {

    let jwt_secret = std::env::var("JWT_SECRET")
        .map_err(|_| AppError::InternalServerError("JWT secret not configured".to_string()))?;
    let key = jwt_secret.as_bytes();

    let password_hash = hash_password(&payload.password)
        .map_err(|e| {
            tracing::error!("Password failed to hash | {}", e);
            AppError::InternalServerError("Something went wrong registering the user.".to_string())
        })?;

    // save user with default role (standard) and status (pending)
    let id = state.db.user.create(
        &payload.name, 
        &payload.email, 
        &password_hash,
        "standard", // Default role
        "pending"   // Default status - requires admin approval
    ).await
        .map_err(|e| {
            tracing::error!("Password failed to save user to DB: | {}", e);
            AppError::InternalServerError("Something went wrong registering the user.".to_string())
        })?;

    let token = generate_token(payload.email, id, key)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        format!(
            "auth_token={}; HttpOnly; Path=/; Max-Age=604800; SameSite=Strict; Secure", 
            token
        ).parse().unwrap()
    );
    
    Ok((headers, StatusCode::OK))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let jwt_secret = std::env::var("JWT_SECRET")
        .map_err(|_| AppError::InternalServerError("JWT secret not configured".to_string()))?;
    let key = jwt_secret.as_bytes();

    let user = state.db.user.find_by_email(&payload.email).await
        .map_err(|e| {
            tracing::error!("Password failed to find user in DB: | {}", e);
            AppError::Unauthorized("Failed to login".to_string())
        })?
        .ok_or_else(|| AppError::Unauthorized("Failed to login".to_string()))?;

    // Verify password
    match is_valid_password(&payload.password, &user.password_hash) {
        false => return Err(AppError::Unauthorized("Failed to login".to_string())),
        _ => ()
    }
    
    // Check if user is approved
    if user.status != "approved" {
        return Err(AppError::Unauthorized("Your account is pending approval from an administrator".to_string()));
    }

    let token = generate_token(payload.email, user.id, key)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        format!(
            "auth_token={}; HttpOnly; Path=/; Max-Age=604800; SameSite=Strict; Secure", 
            token
        ).parse().unwrap()
    );
    
    Ok((headers, StatusCode::OK))
}

pub fn hash_password(password: &str) -> Result<String, password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    
    // Hash the password
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();
    
    Ok(password_hash)
}

pub fn is_valid_password(password: &str, password_hash: &str) -> bool {
    let parsed_hash = match PasswordHash::new(&password_hash) {
        Ok(h) => h,
        Err(_e) => return false
    };

    match Argon2::default().verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => true,
        Err(e) => {
            tracing::error!("Failed to verify password | {}", e);
            false
        }
    }
}

fn generate_token(email: String, user_id: i64, key: &[u8]) -> Result<String, AppError> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(7))
        .expect("valid timestamp")
        .timestamp() as u64;

    let claims = Claims {
        sub: email,
        user_id,
        exp: expiration,
    };

    let header = Header {
        kid: Some("default".to_owned()),
        alg: Algorithm::HS512,
        ..Default::default()
    };

    encode(&header, &claims, &EncodingKey::from_secret(key))
        .map_err(|e| AppError::InternalServerError(e.to_string()))
}
