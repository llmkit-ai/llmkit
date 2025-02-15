use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::{AppError, AppState};

use super::types::{request::prompt_samples::{CreatePromptSampleRequest, UpdatePromptSampleRequest}, response::prompt_samples::PromptSampleResponse};


// Handlers
pub async fn get_sample_by_id(
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> Result<Json<PromptSampleResponse>, AppError> {
    let sample = state.db.prompt_sample.get_by_id(id).await?;
    Ok(Json(sample.into()))
}

pub async fn get_samples_by_prompt(
    Path(prompt_id): Path<i64>,
    State(state): State<AppState>,
) -> Result<Json<Vec<PromptSampleResponse>>, AppError> {
    let samples = state.db.prompt_sample.get_by_prompt(prompt_id).await?;
    Ok(Json(samples.into_iter().map(|s| s.into()).collect()))
}

pub async fn create_sample(
    State(state): State<AppState>,
    Json(request): Json<CreatePromptSampleRequest>,
) -> Result<Json<PromptSampleResponse>, AppError> {
    let sample = state.db.prompt_sample
        .create(
            request.prompt_id,
            sqlx::types::Json(request.input_data),
            request.name,
        )
        .await?;
    
    Ok(Json(sample.into()))
}

pub async fn update_sample(
    Path(id): Path<i64>,
    State(state): State<AppState>,
    Json(request): Json<UpdatePromptSampleRequest>,
) -> Result<Json<PromptSampleResponse>, AppError> {
    let existing = state.db.prompt_sample.get_by_id(id).await?;
    
    let result = state.db.prompt_sample.update(
        existing.id,
        request.input_data.to_string(),
        request.name
    ).await?;

    Ok(Json(result.into()))
}

pub async fn delete_sample(
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    state.db.prompt_sample.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
