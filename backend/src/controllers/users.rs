use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{db::types::user::UserResponse, AppError, AppState};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub user: UserResponse,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub password: Option<String>,
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let user = state
        .db
        .user
        .authenticate_user(&payload.username, &payload.password)
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to authenticate user: {}", e))
        })?;

    match user {
        Some(user) => Ok(Json(LoginResponse { user })),
        None => Err(AppError::Unauthorized("Invalid username or password".to_string())),
    }
}

pub async fn get_users(
    State(state): State<AppState>,
) -> Result<Json<Vec<UserResponse>>, AppError> {
    let users = state.db.user.get_users().await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to get users: {}", e))
    })?;

    Ok(Json(users))
}

pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<UserResponse>, AppError> {
    let user = state.db.user.get_user_by_id(id).await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to get user: {}", e))
    })?;

    match user {
        Some(user) => Ok(Json(user)),
        None => Err(AppError::NotFound(format!("User with id {} not found", id))),
    }
}

pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    let id = state
        .db
        .user
        .create_user(&payload.username, &payload.password, &payload.name)
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to create user: {}", e))
        })?;

    let user = state.db.user.get_user_by_id(id).await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to get created user: {}", e))
    })?;

    match user {
        Some(user) => Ok((StatusCode::CREATED, Json(user))),
        None => Err(AppError::InternalServerError(
            "User was created but could not be retrieved".to_string(),
        )),
    }
}

pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    let success = state
        .db
        .user
        .update_user(
            id,
            payload.name.as_deref(),
            payload.password.as_deref(),
        )
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to update user: {}", e))
        })?;

    if !success {
        return Err(AppError::NotFound(format!("User with id {} not found", id)));
    }

    let user = state.db.user.get_user_by_id(id).await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to get updated user: {}", e))
    })?;

    match user {
        Some(user) => Ok(Json(user)),
        None => Err(AppError::InternalServerError(
            "User was updated but could not be retrieved".to_string(),
        )),
    }
}

pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let success = state.db.user.delete_user(id).await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to delete user: {}", e))
    })?;

    if !success {
        return Err(AppError::NotFound(format!("User with id {} not found", id)));
    }

    Ok(StatusCode::NO_CONTENT)
}