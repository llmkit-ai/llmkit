use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{AppError, AppState};

#[derive(Debug, Serialize)]
pub struct ApiKeyResponse {
    pub id: i64,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct ApiKeyCreateResponse {
    pub id: i64,
    pub name: String,
    pub key: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ApiKeyCreateRequest {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct ApiKeyDeleteResponse {
    pub success: bool,
}

// List all API keys (without the key values)
pub async fn list_api_keys(
    State(state): State<AppState>,
) -> Result<Json<Vec<ApiKeyResponse>>, AppError> {
    let keys = state.db.api_key.get_api_keys().await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to get API keys: {}", e))
    })?;

    let response = keys
        .into_iter()
        .map(|key| {
            ApiKeyResponse {
                id: key.id,
                name: key.name,
                created_at: key.created_at
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_default(),
            }
        })
        .collect();

    Ok(Json(response))
}

// Create a new API key
pub async fn create_api_key(
    State(state): State<AppState>,
    Json(payload): Json<ApiKeyCreateRequest>,
) -> Result<Json<ApiKeyCreateResponse>, AppError> {
    if payload.name.is_empty() {
        return Err(AppError::BadRequest("API key name cannot be empty".to_string()));
    }

    let (id, key) = state.db.api_key.create_api_key(&payload.name).await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to create API key: {}", e))
    })?;

    Ok(Json(ApiKeyCreateResponse {
        id,
        name: payload.name,
        key,
        created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    }))
}

// Delete an API key
pub async fn delete_api_key(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiKeyDeleteResponse>, AppError> {
    let deleted = state.db.api_key.delete_api_key(id).await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to delete API key: {}", e))
    })?;

    if !deleted {
        return Err(AppError::NotFound(format!("API key with ID {} not found", id)));
    }

    Ok(Json(ApiKeyDeleteResponse { success: true }))
}