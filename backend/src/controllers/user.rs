use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher, SaltString
    },
    Argon2, PasswordHash, PasswordVerifier
};

use tower_cookies::{cookie::time::Duration, Cookie, Cookies};

use axum::{
    extract::State, response::IntoResponse, Json
};
use hyper::StatusCode;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

use crate::{middleware::auth::UserId, AppError, AppState};
use super::types::{request::user::{LoginRequest, RegisterRequest}, response::user::MeResponse};


#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    user_id: i64,
    exp: u64,
}

pub async fn register(
    State(state): State<AppState>,
    cookies: Cookies,
    Json(payload): Json<RegisterRequest>
) -> Result<impl IntoResponse, AppError> {
    let use_secure_cookie = std::env::var("USE_SECURE_COOKIE")
        .map_err(|_| {
            tracing::error!("Failed to get USE_SECURE_COOKIE env var");
            return AppError::InternalServerError("Something went wrong with registration.".to_string());
        })?;

    // Check if registration is already completed to provide better error message
    let registration_completed = state.db.user.check_registration_completed().await
        .map_err(|e| {
            tracing::error!("Failed to check registration status | {}", e);
            AppError::InternalServerError("Something went wrong with registration.".to_string())
        })?;
    
    if registration_completed {
        return Err(AppError::Forbidden("Registration is closed. System already has a user account.".to_string()));
    }
    
    // Validate password
    validate_password(&payload.password)?;

    let password_hash = hash_password(&payload.password)
        .map_err(|e| {
            tracing::error!("Password failed to hash | {}", e);
            AppError::InternalServerError("Something went wrong registering the user.".to_string())
        })?;

    // Create the user - this will automatically mark registration as completed
    let id = state.db.user.create(
        &payload.name, 
        &payload.email, 
        &password_hash
    ).await
        .map_err(|e| {
            tracing::error!("Failed to save user to DB: | {}", e);
            AppError::InternalServerError("Something went wrong registering the user.".to_string())
        })?;

    let token = generate_token(payload.email, id, &state.jwt_secret)?;

    let mut cookie = Cookie::new("llmkit_auth_token", token);
    cookie.set_http_only(true);

    if use_secure_cookie.eq("true") {
        cookie.set_secure(true);
    }

    cookie.set_path("/");
    cookie.set_max_age(Duration::days(7));
    cookies.add(cookie);
    
    Ok(StatusCode::OK)
}

pub async fn login(
    State(state): State<AppState>,
    cookies: Cookies,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let use_secure_cookie = std::env::var("USE_SECURE_COOKIE")
        .map_err(|_| {
            tracing::error!("Failed to get USE_SECURE_COOKIE env var");
            return AppError::InternalServerError("Something went wrong with registration.".to_string());
        })?;

    let user = state.db.user.find_by_email(&payload.email).await
        .map_err(|e| {
            tracing::error!("Failed to find user in DB: | {}", e);
            AppError::Unauthorized("Invalid email or password".to_string())
        })?
        .ok_or_else(|| AppError::Unauthorized("Invalid email or password".to_string()))?;

    // Verify password
    match is_valid_password(&payload.password, &user.password_hash) {
        false => return Err(AppError::Unauthorized("Invalid email or password".to_string())),
        _ => ()
    }

    let token = generate_token(payload.email, user.id, &state.jwt_secret)?;

    let mut cookie = Cookie::new("llmkit_auth_token", token);
    cookie.set_http_only(true);

    if use_secure_cookie.eq("true") {
        cookie.set_secure(true);
    }

    cookie.set_path("/");
    cookie.set_max_age(Duration::days(7));
    cookies.add(cookie);
    
    Ok(StatusCode::OK)
}

pub async fn me(
    State(state): State<AppState>,
    request: axum::extract::Request
) -> Result<Json<MeResponse>, AppError> {
    let user_id = request
        .extensions()
        .get::<UserId>()
        .ok_or_else(|| AppError::InternalServerError("Something went wrong finding user".to_string()))?
        .0;

    match state.db.user.find_by_id(user_id).await? {
        Some(u) => {
            Ok(Json(u.into())) 
        },
        None => {
            return Err(AppError::NotFound("Failed to find user".to_string()));
        }
    }
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

fn generate_token(email: String, user_id: i64, secret: &str) -> Result<String, AppError> {
    let key = secret.as_bytes();

    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(7))
        .expect("valid timestamp")
        .timestamp() as u64;

    let claims = Claims {
        sub: email,
        user_id,
        exp: expiration,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(key))
        .map_err(|e| AppError::InternalServerError(e.to_string()))
}

fn validate_password(password: &str) -> Result<(), AppError> {
    // Minimum length check
    if password.len() < 8 {
        return Err(AppError::BadRequest("Password must be at least 8 characters long".to_string()));
    }
    
    // Check for at least one uppercase letter
    if !password.chars().any(|c| c.is_ascii_uppercase()) {
        return Err(AppError::BadRequest("Password must contain at least one uppercase letter".to_string()));
    }
    
    // Check for at least one lowercase letter
    if !password.chars().any(|c| c.is_ascii_lowercase()) {
        return Err(AppError::BadRequest("Password must contain at least one lowercase letter".to_string()));
    }
    
    // Check for at least one digit
    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err(AppError::BadRequest("Password must contain at least one number".to_string()));
    }
    
    // Check for at least one special character
    if !password.chars().any(|c| !c.is_alphanumeric()) {
        return Err(AppError::BadRequest("Password must contain at least one special character".to_string()));
    }
    
    Ok(())
}
