use std::convert::Infallible;

use axum::{
    extract::{Path, State},
    response::sse::{Event, KeepAlive, Sse},
    Json
};
use futures::Stream;
use hyper::StatusCode;
use serde_json::{json, Value};

use tokio::sync::mpsc;

use crate::{services::{llm::Llm, types::llm_props::LlmProps}, AppError, AppState};

use super::types::{request::prompts::{CreatePromptRequest, UpdatePromptRequest}, response::prompts::{PromptExecutionResponse, PromptResponse}};


pub async fn create_prompt(
    State(state): State<AppState>,
    Json(payload): Json<CreatePromptRequest>,
) -> Result<Json<PromptResponse>, AppError> {
    let id = state.db.prompt.create_prompt(
        &payload.key, 
        &payload.system, 
        &payload.user, 
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
        &payload.system, 
        &payload.user, 
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

#[axum::debug_handler]
pub async fn execute_prompt(
    Path(id): Path<i64>,
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<Json<PromptExecutionResponse>, AppError> {
    let prompt = match state.prompt_cache.get(&id).await {
        Some(p) => p,
        None => { 
            let prompt = state.db.prompt.get_prompt(id).await?;
            state.prompt_cache.insert(id, prompt.clone()).await;
            prompt
        }
    };

    let llm_props = LlmProps::new(prompt.clone(), payload).map_err(|e| {
        tracing::error!("{}", e);
        AppError::InternalServerError("An error occured processing prompt template".to_string())
    })?;

    let llm = Llm::new(llm_props, state.db.log.clone());

    let res = match prompt.json_mode {
        true => {
            let res = llm.json()
                .await
                .map_err(|_| AppError::InternalServerError("Something went wrong".to_string()))?;

            res
        },
        false => {
            let res = llm.text()
                .await
                .map_err(|_| AppError::InternalServerError("Something went wrong".to_string()))?;

            res
        }
    };

    let log = state.db.log.get_log_by_id(res.log_id).await?.ok_or_else(|| AppError::NotFound("Log not found".to_string()))?;

    Ok(Json(PromptExecutionResponse::from_log_row(res.content, log)))
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
    // Create oneshot channel for log ID
    let (stream_res_tx, stream_res_rx) = tokio::sync::oneshot::channel();
    
    let llm_props = LlmProps::new(prompt.clone(), payload).map_err(|e| {
        tracing::error!("{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let llm = Llm::new(llm_props, state.db.log);

    tokio::spawn(async move {
        let result = llm.stream(tx).await;

        if let Ok(llm_stream_response) = result {
            let _ = stream_res_tx.send(llm_stream_response.log_id);
        }
    });

    let stream = async_stream::stream! {
        // Process regular stream messages
        while let Some(result) = rx.recv().await {
            match result {
                Ok(content) => {
                    let event = Event::default().data(content.clone());
                    if content.contains("[DONE]") || content.contains("Done:") {
                        break;
                    }

                    yield Ok(event);
                }
                Err(e) => {
                    eprintln!("error in stream: {:?}", e);
                }
            }
        }

        // Wait for and send log ID after stream completes
        match stream_res_rx.await {
            Ok(log_id) => {
                let log_event = json!({ "log_id": log_id });
                yield Ok(Event::default().data(log_event.to_string()));
            },
            Err(_) => eprintln!("Failed to receive log ID"),
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}
