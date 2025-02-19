use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::{AppError, AppState};

use super::types::{request::prompt_eval::{CreateEvalTestRequest, UpdateEvalTestRequest}, response::prompt_eval::PromptEvalResponse};


// Handlers
pub async fn get_eval_test_by_id(
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> Result<Json<PromptEvalResponse>, AppError> {
    let sample = state.db.prompt_eval.get_by_id(id).await?;
    Ok(Json(sample.into()))
}

pub async fn get_eval_test_by_prompt(
    Path(prompt_id): Path<i64>,
    State(state): State<AppState>,
) -> Result<Json<Vec<PromptEvalResponse>>, AppError> {
    let samples = state.db.prompt_eval.get_by_prompt(prompt_id).await?;
    Ok(Json(samples.into_iter().map(|s| s.into()).collect()))
}

pub async fn create_eval_test(
    State(state): State<AppState>,
    Json(request): Json<CreateEvalTestRequest>,
) -> Result<Json<PromptEvalResponse>, AppError> {
    let sample = state.db.prompt_eval
        .create(
            request.prompt_id,
            sqlx::types::Json(request.input_data),
            "human",
            request.name,
        )
        .await?;
    
    Ok(Json(sample.into()))
}

pub async fn update_eval_test(
    Path(id): Path<i64>,
    State(state): State<AppState>,
    Json(request): Json<UpdateEvalTestRequest>,
) -> Result<Json<PromptEvalResponse>, AppError> {
    let existing = state.db.prompt_eval.get_by_id(id).await?;
    
    let result = state.db.prompt_eval.update(
        existing.id,
        request.input_data.to_string(),
        request.name
    ).await?;

    Ok(Json(result.into()))
}

pub async fn delete_eval_test(
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    state.db.prompt_eval.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
