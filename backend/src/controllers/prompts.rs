use core::panic;
use std::convert::Infallible;

use axum::{
    extract::{Path, State},
    response::sse::{Event, KeepAlive, Sse},
    Json,
};
use futures::Stream;
use hyper::StatusCode;
use serde_json::Value;

use tokio::sync::mpsc;

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

    // add the new prompt to the cache
    state.prompt_cache.insert(id, prompt.clone()).await;

    Ok(Json(prompt.into()))
}

pub async fn get_prompt(
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> Result<Json<PromptResponse>, AppError> {
    let prompt = match state.prompt_cache.get(&id).await {
        Some(p) => p,
        None => { 
            let prompt = state.db.prompt.get_prompt(id).await?;
            state.prompt_cache.insert(id, prompt.clone()).await;
            prompt
        }
    };

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

    state.prompt_cache.insert(id, prompt.clone()).await;

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

    state.prompt_cache.remove(&id).await;

    Ok(())
}

pub async fn execute_prompt(
    Path(id): Path<i64>,
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<String, AppError> {
    let prompt = match state.prompt_cache.get(&id).await {
        Some(p) => p,
        None => { 
            let prompt = state.db.prompt.get_prompt(id).await?;
            state.prompt_cache.insert(id, prompt.clone()).await;
            prompt
        }
    };

    let llm_props = LlmProps::new(prompt.clone(), payload);
    let llm = Llm::new(llm_props);

    match prompt.json_mode {
        true => {
            let res = llm.json()
                .await
                .map_err(|_| AppError::InternalServerError("Something went wrong".to_string()))?;

            let res = serde_json::to_string(&res)
                .map_err(|_| AppError::InternalServerError("Something went wrong".to_string()))?;

            Ok(res)
        },
        false => {
            match llm.text().await {
                Ok(t) => Ok(t),
                Err(e) => {
                    println!("error: {}", e);
                    return Err(AppError::InternalServerError("Something went wrong".to_string()));
                }
            }
        }
    }
}

pub async fn execute_prompt_stream(
    Path(id): Path<i64>,
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, StatusCode> {
    let prompt = match state.prompt_cache.get(&id).await {
        Some(p) => p,
        None => {
            let prompt = state.db.prompt.get_prompt(id).await.unwrap();
            state.prompt_cache.insert(id, prompt.clone()).await;
            prompt
        }
    };

    let (tx, mut rx) = mpsc::channel(100);
    let llm_props = LlmProps::new(prompt.clone(), payload);
    let llm = Llm::new(llm_props);

    tokio::spawn(async move {
        if let Err(e) = llm.stream(tx).await {
            eprintln!("LLM streaming error: {}", e);
        }
    });

    let stream = async_stream::stream! {
        while let Some(result) = rx.recv().await {
            match result {
                Ok(content) => {
                    let event = Event::default().data(content.clone());
                    if content == "[DONE]" {
                        break;
                    }
                    
                    yield Ok(event)
                }
                Err(e) => {
                    eprintln!("error in stream: {:?}", e);
                    panic!()
                }
            }
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}
