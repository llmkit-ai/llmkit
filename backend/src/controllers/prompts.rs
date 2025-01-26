use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::Value;

use crate::{services::{llm::Llm, types::llm_props::LlmProps}, AppError, AppState};

use super::types::{request::prompts::{CreatePromptRequest, UpdatePromptRequest}, response::prompts::PromptResponse};


pub async fn create_prompt(
    State(state): State<AppState>,
    Json(payload): Json<CreatePromptRequest>,
) -> Result<Json<PromptResponse>, AppError> {
    let id = state.db.prompt.create_prompt(
        &payload.key, 
        &payload.prompt, 
        payload.model_id,
        payload.max_tokens,
        payload.temperature,
        payload.json_mode
    ).await?;
    let prompt = state.db.prompt.get_prompt(id).await
        .map_err(|_| AppError::NotFound("Prompt not found after creation".into()))?;
    Ok(Json(prompt.into()))
}

pub async fn get_prompt(
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> Result<Json<PromptResponse>, AppError> {
    let prompt = state.db.prompt.get_prompt(id).await
        .map_err(|_| AppError::NotFound("Prompt not found".into()))?;
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
    let updated = state.db.prompt.update_prompt(
        id, 
        &payload.key, 
        &payload.prompt, 
        payload.model_id,
        payload.max_tokens,
        payload.temperature,
        payload.json_mode
    ).await?;
    if !updated {
        return Err(AppError::NotFound("Prompt not found".into()));
    }
    let prompt = state.db.prompt.get_prompt(id).await
        .map_err(|_| AppError::NotFound("Prompt not found after update".into()))?;

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

pub async fn execute_prompt(
    Path(id): Path<i64>,
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<String, AppError> {
    let prompt = state.db.prompt.get_prompt(id).await?;
    let llm_props = LlmProps::from_prompt(prompt, payload);
    let llm = Llm::new(llm_props).map_err(|_| AppError::InternalServerError("Something went wrong".to_string()))?;

    let res = llm.text()
        .await
        .map_err(|_| AppError::InternalServerError("Something went wrong".to_string()))?;

    Ok(res)
}
