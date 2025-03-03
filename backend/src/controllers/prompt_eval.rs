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
    // For system input, serialize the JSON Value to a string if present
    let system_input = request.system_prompt_input.map(|val| val.to_string());
    
    // For user input, either use it as JSON or as plain text depending on the prompt type
    let user_input = request.user_prompt_input.to_string();
    
    let sample = state.db.prompt_eval
        .create(
            request.prompt_id,
            system_input,
            user_input,
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
    
    // For system input, serialize the JSON Value to a string if present
    let system_input = request.system_prompt_input.map(|val| val.to_string());
    let user_input = request.user_prompt_input;
    
    let result = state.db.prompt_eval.update(
        existing.id,
        system_input,
        user_input,
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
