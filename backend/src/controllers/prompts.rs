use axum::{
    extract::{Path, State},
    response::sse::{Event, KeepAlive, Sse},
    Json,
};
use futures::Stream;
use hyper::StatusCode;
use serde_json::json;
use std::convert::Infallible;
use tokio::sync::mpsc;

use crate::{
    services::{
        llm_v2::Llm,
        types::{
            chat_request::{LlmServiceRequest, Message},
            chat_response::LlmServiceChatCompletionResponse,
        },
    },
    AppError, AppState,
};

use super::types::{
    request::prompts::{ApiCompletionRequest, CreatePromptRequest, UpdatePromptRequest},
    response::prompts::PromptResponse,
};

pub async fn create_prompt(
    State(state): State<AppState>,
    Json(payload): Json<CreatePromptRequest>,
) -> Result<Json<PromptResponse>, AppError> {
    let id = state
        .db
        .prompt
        .create_prompt(
            &payload.key,
            &payload.system,
            &payload.user,
            payload.model_id,
            payload.max_tokens,
            payload.temperature,
            payload.json_mode,
            payload.json_schema.as_deref(),
            &payload.prompt_type,
            payload.is_chat,
        )
        .await?;
    let prompt = state
        .db
        .prompt
        .get_prompt(id)
        .await
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
    let updated = state
        .db
        .prompt
        .update_prompt(
            id,
            &payload.key,
            &payload.system,
            &payload.user,
            payload.model_id,
            payload.max_tokens,
            payload.temperature,
            payload.json_mode,
            payload.json_schema.as_deref(),
            &payload.prompt_type,
            payload.is_chat,
        )
        .await?;

    if !updated {
        return Err(AppError::NotFound("Prompt not found".into()));
    }

    let prompt = state
        .db
        .prompt
        .get_prompt(id)
        .await
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

// OpenAI compatible API endpoints
#[axum::debug_handler]
pub async fn api_completions(
    State(state): State<AppState>,
    Json(payload): Json<ApiCompletionRequest>,
) -> Result<Json<LlmServiceChatCompletionResponse>, AppError> {
    // Look up the prompt by key (model field in the request)
    let prompt_key = &payload.model;
    let prompt = state
        .db
        .prompt
        .get_prompt_by_key(prompt_key)
        .await
        .map_err(|_| AppError::NotFound(format!("Prompt with key '{}' not found", prompt_key)))?;

    // Insert into cache
    state.prompt_cache.insert(prompt.id, prompt.clone()).await;

    // Handle template rendering differently based on prompt type and payload
    let llm_props = if prompt.prompt_type == "static" || prompt.prompt_type == "dynamic_system" {
        // For static or dynamic_system, we render the system prompt with context
        // and use the messages array
        if payload.messages.is_empty() {
            return Err(AppError::BadRequest(
                "Messages array cannot be empty".into(),
            ));
        }

        // Extract context from the system message if it exists
        let context = payload
            .messages
            .iter()
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

        LlmServiceRequest::new_chat(prompt.clone(), context, payload.messages.clone()).map_err(
            |e| {
                tracing::error!("{}", e);
                AppError::InternalServerError(
                    "An error occurred processing chat prompt".to_string(),
                )
            },
        )?
    } else if prompt.prompt_type == "dynamic_both" {
        // For dynamic_both, we pass the entire request as context
        let system_context = payload
            .messages
            .iter()
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

        let user_context = payload
            .messages
            .iter()
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

        LlmServiceRequest::new_split_context(prompt.clone(), system_context, user_context).map_err(
            |e| {
                tracing::error!("{}", e);
                AppError::InternalServerError(
                    "An error occurred processing prompt template".to_string(),
                )
            },
        )?
    } else {
        return Err(AppError::BadRequest(format!(
            "Unsupported prompt type: {}",
            prompt.prompt_type
        )));
    };

    // Apply request overrides if specified
    let llm_props = if let Some(max_tokens) = payload.max_tokens {
        LlmServiceRequest {
            max_tokens,
            ..llm_props
        }
    } else {
        llm_props
    };

    let llm_props = if let Some(temperature) = payload.temperature {
        LlmServiceRequest {
            temperature,
            ..llm_props
        }
    } else {
        llm_props
    };

    // Set JSON mode if specified in request format
    let json_mode = if let Some(ref format) = payload.response_format {
        format.r#type == "json_object"
    } else {
        false
    };

    let llm_props = LlmServiceRequest {
        json_mode,
        ..llm_props
    };

    let llm = Llm::new(llm_props.clone(), state.db.log.clone());

    let res = if llm_props.json_mode {
        llm.json().await.map_err(|e| {
            tracing::error!("{}", e);
            let error: AppError = e.into();
            return error;
        })?
    } else {
        llm.text().await.map_err(|e| {
            tracing::error!("{}", e);
            let error: AppError = e.into();
            return error;
        })?
    };

    Ok(Json(res.0))
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
        let context = payload
            .messages
            .iter()
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

        match LlmServiceRequest::new_chat(prompt.clone(), context, payload.messages.clone()) {
            Ok(props) => props,
            Err(e) => {
                tracing::error!("{}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    } else if prompt.prompt_type == "dynamic_both" {
        // For dynamic_both, we pass the entire request as context
        // Extract context from the system message if it exists
        let system_context = payload
            .messages
            .iter()
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

        let user_context = payload
            .messages
            .iter()
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

        match LlmServiceRequest::new_split_context(prompt.clone(), system_context, user_context) {
            Ok(props) => props,
            Err(e) => {
                tracing::error!("{}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    } else {
        return Err(StatusCode::BAD_REQUEST);
    };

    // Apply request overrides if specified
    let llm_props = if let Some(max_tokens) = payload.max_tokens {
        LlmServiceRequest {
            max_tokens,
            ..llm_props
        }
    } else {
        llm_props
    };

    let llm_props = if let Some(temperature) = payload.temperature {
        LlmServiceRequest {
            temperature,
            ..llm_props
        }
    } else {
        llm_props
    };

    // Set JSON mode if specified in request format
    let json_mode = if let Some(ref format) = payload.response_format {
        format.r#type == "json_object"
    } else {
        false
    };

    let llm_props = LlmServiceRequest {
        json_mode,
        ..llm_props
    };

    let (tx, mut rx) = mpsc::channel(100);
    let llm = Llm::new(llm_props.clone(), state.db.log);

    tokio::spawn(async move {
        let _ = llm.stream(tx).await;
    });

    let stream = async_stream::stream! {
        // Process regular stream messages
        while let Some(result) = rx.recv().await {
            match result {
                Ok(content) => {
                    if content.is_done_sentinel() {
                        yield Ok(Event::default().data(serde_json::to_string(&content).unwrap()));
                        break;
                    }

                    yield Ok(Event::default().data(serde_json::to_string(&content).expect("Failed to turn chunk into string")));
                }
                Err(e) => {
                    tracing::error!("error in stream: {:?}", e);
                }
            }
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}
