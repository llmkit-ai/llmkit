use axum::{
    extract::{Path, State},
    response::sse::{Event, KeepAlive, Sse},
    Json
};
use chrono::Utc;
use futures::Stream;
use hyper::StatusCode;
use serde_json::{json, Value};
use std::convert::Infallible;
use tokio::sync::mpsc;
use uuid;

use crate::{services::{llm::Llm, types::{llm_props::LlmProps, message::Message}}, AppError, AppState};

use super::types::{
    request::prompts::{CreatePromptRequest, UpdatePromptRequest, ApiCompletionRequest}, 
    response::prompts::{PromptExecutionResponse, PromptResponse, ApiCompletionResponse, ApiCompletionChunk, ApiChunkChoice, ApiDelta}
};


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
        payload.json_mode,
        &payload.prompt_type,
        payload.is_chat
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
        payload.json_mode,
        &payload.prompt_type,
        payload.is_chat
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
                    tracing::error!("error in stream: {:?}", e);
                }
            }
        }

        // Wait for and send log ID after stream completes
        match stream_res_rx.await {
            Ok(log_id) => {
                let log_event = json!({ "log_id": log_id });
                yield Ok(Event::default().data(log_event.to_string()));
            },
            Err(_) => tracing::error!("Failed to receive log ID"),
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

// OpenAI compatible API endpoints
#[axum::debug_handler]
pub async fn api_completions(
    State(state): State<AppState>,
    Json(payload): Json<ApiCompletionRequest>,
) -> Result<Json<ApiCompletionResponse>, AppError> {
    // Look up the prompt by key (model field in the request)
    let prompt_key = &payload.model;
    let prompt = state.db.prompt.get_prompt_by_key(prompt_key).await
        .map_err(|_| AppError::NotFound(format!("Prompt with key '{}' not found", prompt_key)))?;
    
    // Insert into cache
    state.prompt_cache.insert(prompt.id, prompt.clone()).await;
    
    // Handle template rendering differently based on prompt type and payload
    let llm_props = if prompt.prompt_type == "static" || prompt.prompt_type == "dynamic_system" {
        // For static or dynamic_system, we render the system prompt with context
        // and use the messages array
        if payload.messages.is_empty() {
            return Err(AppError::BadRequest("Messages array cannot be empty".into()));
        }
        
        // Extract context from the system message if it exists
        let context = payload.messages.iter()
            .find(|msg| matches!(msg, Message::System { .. }))
            .and_then(|msg| {
                if let Message::System { content } = msg {
                    // Try to parse content as JSON, or use empty object if it fails
                    serde_json::from_str::<serde_json::Value>(content).ok()
                } else {
                    None
                }
            })
            .unwrap_or(json!({}));

        
        LlmProps::new_chat(
            prompt.clone(), 
            context, 
            payload.messages.clone()
        ).map_err(|e| {
            tracing::error!("{}", e);
            AppError::InternalServerError("An error occurred processing chat prompt".to_string())
        })?
    } else if prompt.prompt_type == "dynamic_both" {
        // For dynamic_both, we pass the entire request as context
        let system_context = payload.messages.iter()
            .find(|msg| matches!(msg, Message::System { .. }))
            .and_then(|msg| {
                if let Message::System { content } = msg {
                    // Try to parse content as JSON, or use empty object if it fails
                    serde_json::from_str::<serde_json::Value>(content).ok()
                } else {
                    None
                }
            })
            .unwrap_or(json!({}));

        let user_context = payload.messages.iter()
            .find(|msg| matches!(msg, Message::User { .. }))
            .and_then(|msg| {
                if let Message::User { content } = msg {
                    // Try to parse content as JSON, or use empty object if it fails
                    serde_json::from_str::<serde_json::Value>(content).ok()
                } else {
                    None
                }
            })
            .unwrap_or(json!({}));

        LlmProps::new_split_context(prompt.clone(), system_context, user_context).map_err(|e| {
            tracing::error!("{}", e);
            AppError::InternalServerError("An error occurred processing prompt template".to_string())
        })?
    } else {
        return Err(AppError::BadRequest(format!("Unsupported prompt type: {}", prompt.prompt_type)));
    };
    
    // Apply request overrides if specified
    let llm_props = if let Some(max_tokens) = payload.max_tokens {
        LlmProps { max_tokens, ..llm_props }
    } else {
        llm_props
    };
    
    let llm_props = if let Some(temperature) = payload.temperature {
        LlmProps { temperature, ..llm_props }
    } else {
        llm_props
    };
    
    // Set JSON mode if specified in request format
    let json_mode = if let Some(ref format) = payload.response_format {
        format.r#type == "json_object"
    } else {
        false
    };
    
    let llm_props = LlmProps {
        json_mode,
        ..llm_props
    };
    
    let llm = Llm::new(llm_props.clone(), state.db.log.clone());
    
    let res = if llm_props.json_mode {
        llm.json()
            .await
            .map_err(|_| AppError::InternalServerError("Something went wrong".to_string()))?
    } else {
        llm.text()
            .await
            .map_err(|_| AppError::InternalServerError("Something went wrong".to_string()))?
    };
    
    let log = state.db.log.get_log_by_id(res.log_id).await?
        .ok_or_else(|| AppError::NotFound("Log not found".to_string()))?;
    
    let execution_response = PromptExecutionResponse::from_log_row(res.content, log);
    
    // Convert to OpenAI-compatible response
    let api_response = execution_response.to_api_response(prompt_key);
    
    Ok(Json(api_response))
}

#[axum::debug_handler]
pub async fn api_completions_stream(
    State(state): State<AppState>,
    Json(payload): Json<ApiCompletionRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, StatusCode> {
    // Look up the prompt by key (model field in the request)
    let prompt_key = &payload.model;
    let prompt = match state.db.prompt.get_prompt_by_key(prompt_key).await {
        Ok(p) => p,
        Err(_) => return Err(StatusCode::NOT_FOUND),
    };
    
    // Insert into cache
    state.prompt_cache.insert(prompt.id, prompt.clone()).await;
    
    // Handle template rendering differently based on prompt type and payload
    let llm_props = if prompt.prompt_type == "static" || prompt.prompt_type == "dynamic_system" {
        // For static or dynamic_system, we render the system prompt with context
        // and use the messages array for chat history
        if payload.messages.is_empty() {
            return Err(StatusCode::BAD_REQUEST);
        }
        
        // Extract context from the system message if it exists
        let context = payload.messages.iter()
            .find(|msg| matches!(msg, Message::System { .. }))
            .and_then(|msg| {
                if let Message::System { content } = msg {
                    // Try to parse content as JSON, or use empty object if it fails
                    serde_json::from_str::<serde_json::Value>(content).ok()
                } else {
                    None
                }
            })
            .unwrap_or(json!({}));
        
        match LlmProps::new_chat(
            prompt.clone(), 
            context, 
            payload.messages.clone()
        ) {
            Ok(props) => props,
            Err(e) => {
                tracing::error!("{}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    } else if prompt.prompt_type == "dynamic_both" {
        // For dynamic_both, we pass the entire request as context
        // Extract context from the system message if it exists
        let system_context = payload.messages.iter()
            .find(|msg| matches!(msg, Message::System { .. }))
            .and_then(|msg| {
                if let Message::System { content } = msg {
                    // Try to parse content as JSON, or use empty object if it fails
                    serde_json::from_str::<serde_json::Value>(content).ok()
                } else {
                    None
                }
            })
            .unwrap_or(json!({}));

        let user_context = payload.messages.iter()
            .find(|msg| matches!(msg, Message::User { .. }))
            .and_then(|msg| {
                if let Message::User { content } = msg {
                    // Try to parse content as JSON, or use empty object if it fails
                    serde_json::from_str::<serde_json::Value>(content).ok()
                } else {
                    None
                }
            })
            .unwrap_or(json!({}));

        match LlmProps::new_split_context(prompt.clone(), system_context, user_context) {
            Ok(props) => props,
            Err(e) => {
                tracing::error!("{}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    } else {
        return Err(StatusCode::BAD_REQUEST);
    };

    tracing::info!("messages: {:?}", payload.messages);
    tracing::info!("llm_props: {:?}", llm_props);
    
    // Apply request overrides if specified
    let llm_props = if let Some(max_tokens) = payload.max_tokens {
        LlmProps { max_tokens, ..llm_props }
    } else {
        llm_props
    };
    
    let llm_props = if let Some(temperature) = payload.temperature {
        LlmProps { temperature, ..llm_props }
    } else {
        llm_props
    };
    
    // Set JSON mode if specified in request format
    let json_mode = if let Some(ref format) = payload.response_format {
        format.r#type == "json_object"
    } else {
        false
    };
    
    let llm_props = LlmProps {
        json_mode,
        ..llm_props
    };
    
    let (tx, mut rx) = mpsc::channel(100);
    // Create oneshot channel for log ID
    let (stream_res_tx, stream_res_rx) = tokio::sync::oneshot::channel();
    
    let llm = Llm::new(llm_props.clone(), state.db.log);
    
    // Generate a unique ID for this streaming session
    let stream_id = format!("chatcmpl-{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
    let prompt_key_string = prompt_key.to_string();
    let prompt_key_for_stream = prompt_key_string.clone();
    
    tokio::spawn(async move {
        let result = llm.stream(tx).await;
        
        if let Ok(llm_stream_response) = result {
            let _ = stream_res_tx.send((llm_stream_response.log_id, prompt_key_for_stream));
        }
    });
    
    let stream = async_stream::stream! {
        let created_time = Utc::now().timestamp();
        let mut is_first_chunk = true;
        let mut content_buffer = String::new();
        
        // Process regular stream messages
        while let Some(result) = rx.recv().await {
            match result {
                Ok(content) => {
                    if content.contains("[DONE]") || content.contains("Done:") {
                        // Send final chunk with finish_reason
                        let final_chunk = ApiCompletionChunk {
                            id: stream_id.clone(),
                            object: "chat.completion.chunk".to_string(),
                            created: created_time,
                            model: prompt_key_string.clone(),
                            choices: vec![
                                ApiChunkChoice {
                                    index: 0,
                                    delta: ApiDelta {
                                        content: None,
                                        role: None,
                                    },
                                    finish_reason: Some("stop".to_string()),
                                }
                            ],
                        };
                        
                        yield Ok(Event::default().data(serde_json::to_string(&final_chunk).unwrap()));
                        yield Ok(Event::default().data("[DONE]"));
                        break;
                    }
                    
                    // For first chunk, include the role
                    if is_first_chunk {
                        let first_chunk = ApiCompletionChunk {
                            id: stream_id.clone(),
                            object: "chat.completion.chunk".to_string(),
                            created: created_time,
                            model: prompt_key_string.clone(),
                            choices: vec![
                                ApiChunkChoice {
                                    index: 0,
                                    delta: ApiDelta {
                                        content: None,
                                        role: Some("assistant".to_string()),
                                    },
                                    finish_reason: None,
                                }
                            ],
                        };
                        
                        yield Ok(Event::default().data(serde_json::to_string(&first_chunk).unwrap()));
                        is_first_chunk = false;
                    }
                    
                    // Regular content chunk
                    content_buffer.push_str(&content);
                    
                    let content_chunk = ApiCompletionChunk {
                        id: stream_id.clone(),
                        object: "chat.completion.chunk".to_string(),
                        created: created_time,
                        model: prompt_key_string.clone(),
                        choices: vec![
                            ApiChunkChoice {
                                index: 0,
                                delta: ApiDelta {
                                    content: Some(content),
                                    role: None,
                                },
                                finish_reason: None,
                            }
                        ],
                    };
                    
                    yield Ok(Event::default().data(serde_json::to_string(&content_chunk).unwrap()));
                }
                Err(e) => {
                    tracing::error!("error in stream: {:?}", e);
                }
            }
        }
        
        // Wait for and send log ID after stream completes
        match stream_res_rx.await {
            Ok((_log_id, _prompt_key)) => {
                // Already sent the [DONE] event above
            },
            Err(_) => tracing::error!("Failed to receive log ID"),
        }
    };
    
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}
