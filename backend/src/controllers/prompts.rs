use axum::{
    extract::{Path, State},
    Json,
};
use crate::{
    models::{
        request::prompts::{
            CreatePromptRequest, 
            UpdatePromptRequest
        }, 
        response::prompts::PromptResponse
    }, 
    AppError, 
    AppState
};


pub async fn create_prompt(
    State(state): State<AppState>,
    Json(payload): Json<CreatePromptRequest>,
) -> Result<Json<PromptResponse>, AppError> {
    let id = state.db.prompt.create_prompt(&payload.key, &payload.prompt, &payload.model).await?;
    let prompt = state.db.prompt.get_prompt(id).await?
        .ok_or(AppError::NotFound("Prompt not found after creation".into()))?;
    Ok(Json(prompt.into()))
}

pub async fn get_prompt(
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> Result<Json<PromptResponse>, AppError> {
    let prompt = state.db.prompt.get_prompt(id).await?
        .ok_or(AppError::NotFound("Prompt not found".into()))?;
    Ok(Json(prompt.into()))
}

pub async fn list_prompts(
    State(state): State<AppState>,
) -> Result<Json<Vec<PromptResponse>>, AppError> {
    let prompts = state.db.prompt.list_prompts().await?;
    Ok(Json(prompts.into_iter().map(|p| p.into()).collect()))
}

pub async fn update_prompt(
    Path(id): Path<i64>,
    State(state): State<AppState>,
    Json(payload): Json<UpdatePromptRequest>,
) -> Result<Json<PromptResponse>, AppError> {
    let updated = state.db.prompt.update_prompt(id, &payload.key, &payload.prompt, &payload.model).await?;
    if !updated {
        return Err(AppError::NotFound("Prompt not found".into()));
    }
    let prompt = state.db.prompt.get_prompt(id).await?
        .ok_or(AppError::NotFound("Prompt not found after update".into()))?;
    Ok(Json(prompt.into()))
}

pub async fn delete_prompt(
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> Result<(), AppError> {
    let deleted = state.db.prompt.delete_prompt(id).await?;
    if !deleted {
        return Err(AppError::NotFound("Prompt not found".into()));
    }
    Ok(())
}
