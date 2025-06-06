use axum::{
    extract::{Path, State},
    Json,
};

use super::types::{
    request::models::{CreateModelRequest, UpdateModelRequest},
    response::models::ModelResponse,
};
use crate::{AppError, AppState};

pub async fn list_models(
    State(state): State<AppState>,
) -> Result<Json<Vec<ModelResponse>>, AppError> {
    let models = state.db.model.list_models().await?;
    Ok(Json(models.into_iter().map(|m| m.into()).collect()))
}

pub async fn create_model(
    State(state): State<AppState>,
    Json(payload): Json<CreateModelRequest>,
) -> Result<Json<ModelResponse>, AppError> {
    let model_id = state
        .db
        .model
        .create_model(
            payload.provider_id,
            &payload.name,
            payload.supports_json,
            payload.supports_json_schema,
            payload.supports_tools,
            payload.is_reasoning
        )
        .await?;

    let model = state
        .db
        .model
        .get_model_by_id(model_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Model not found after creation".to_string()))?;

    Ok(Json(model.into()))
}

pub async fn update_model(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateModelRequest>,
) -> Result<Json<ModelResponse>, AppError> {
    let updated = state
        .db
        .model
        .update_model(
            id,
            payload.provider_id,
            &payload.name,
            payload.supports_json,
            payload.supports_json_schema,
            payload.supports_tools,
            payload.is_reasoning
        )
        .await?;

    if !updated {
        return Err(AppError::NotFound("Model not found".to_string()));
    }

    let model = state
        .db
        .model
        .get_model_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("Model not found after update".to_string()))?;

    Ok(Json(model.into()))
}
