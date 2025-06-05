use axum::{
    extract::{Path, State},
    response::{sse::{Event, KeepAlive, Sse}, IntoResponse, Response},
    Json,
};
use futures::Stream;
use std::{convert::Infallible, pin::Pin};
use tokio::sync::mpsc;

use crate::{
    common::types::{chat_request::{
        ChatCompletionRequest, ChatCompletionRequestFunctionDescription, ChatCompletionRequestTool
    }, chat_response::LlmServiceChatCompletionResponse}, 
    services::{
        llm::Llm,
        types::llm_service::LlmServiceRequest,
    }, 
    AppError, 
    AppState
};

use super::types::{
    request::prompts::{CreatePromptRequest, UpdatePromptRequest},
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

    // Fetch associated tools (newly created prompt will have none, but including for consistency)
    let tools = state.db.tool.get_tools_by_prompt_version(prompt.version_id).await?;
    
    // Convert prompt to PromptResponse
    let mut response: PromptResponse = prompt.into();
    
    // Add tools to the response
    response.tools = tools.into_iter().map(|t| t.into()).collect();

    Ok(Json(response))
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
    
    // Fetch associated tools
    let tools = state.db.tool.get_tools_by_prompt_version(prompt.version_id).await?;
    
    // Convert prompt to PromptResponse
    let mut response: PromptResponse = prompt.into();
    
    // Add tools to the response
    response.tools = tools.into_iter().map(|t| t.into()).collect();
    
    Ok(Json(response))
}

pub async fn list_prompts(
    State(state): State<AppState>,
) -> Result<Json<Vec<PromptResponse>>, AppError> {
    let prompts = state.db.prompt.list_prompts().await?;
    
    let mut responses = Vec::new();
    for prompt in prompts {
        // Fetch associated tools for each prompt
        let tools = state.db.tool.get_tools_by_prompt_version(prompt.version_id).await?;
        
        // Convert prompt to PromptResponse
        let mut response: PromptResponse = prompt.into();
        
        // Add tools to the response
        response.tools = tools.into_iter().map(|t| t.into()).collect();
        
        responses.push(response);
    }
    
    Ok(Json(responses))
}

pub async fn update_prompt(
    Path(id): Path<i64>,
    State(state): State<AppState>,
    Json(payload): Json<UpdatePromptRequest>,
) -> Result<Json<PromptResponse>, AppError> {
    // Get the current prompt to access its version ID before update
    let current_prompt = state
        .db
        .prompt
        .get_prompt(id)
        .await?;
    
    // Get the tools associated with the current prompt version
    let current_tools = state.db.tool.get_tools_by_prompt_version(current_prompt.version_id).await?;
    
    // Update the prompt, which creates a new version
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

    // Get the updated prompt with its new version ID
    let prompt = state
        .db
        .prompt
        .get_prompt(id)
        .await
        .map_err(|_| AppError::NotFound("Prompt not found after update".into()))?;

    // Add to cache
    state.prompt_cache.insert(id, prompt.clone()).await;

    // Copy tool associations from the previous version to the new version
    for tool in current_tools {
        state.db.tool.associate_tool_with_prompt_version(tool.id, prompt.version_id).await?;
    }

    // Fetch associated tools for the new version
    let tools = state.db.tool.get_tools_by_prompt_version(prompt.version_id).await?;
    
    // Convert prompt to PromptResponse
    let mut response: PromptResponse = prompt.into();
    
    // Add tools to the response
    response.tools = tools.into_iter().map(|t| t.into()).collect();

    Ok(Json(response))
}

pub async fn delete_prompt(
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> Result<(), AppError> {
    // Get the prompt before deletion to access its data
    let prompt = match state.db.prompt.get_prompt(id).await {
        Ok(p) => p,
        Err(_) => return Err(AppError::NotFound("Prompt not found".into()))
    };
    
    tracing::info!("Deleting prompt ID: {}, version ID: {}", id, prompt.version_id);
    
    // Delete the prompt
    let deleted = state.db.prompt.delete_prompt(id).await?;

    if !deleted {
        return Err(AppError::NotFound("Prompt not found".into()));
    }

    // Remove from cache
    state.prompt_cache.remove(&id).await;

    tracing::info!("Prompt {} deleted successfully", id);
    Ok(())
}

pub async fn get_prompt_versions(
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> Result<Json<Vec<PromptResponse>>, AppError> {
    let versions = state.db.prompt.get_prompt_versions(id).await?;
    
    let mut responses = Vec::new();
    for version in versions {
        // Fetch associated tools for each version
        let tools = state.db.tool.get_tools_by_prompt_version(version.version_id).await?;
        
        // Convert version to PromptResponse
        let mut response: PromptResponse = version.into();
        
        // Add tools to the response
        response.tools = tools.into_iter().map(|t| t.into()).collect();
        
        responses.push(response);
    }
    
    Ok(Json(responses))
}

pub async fn set_active_version(
    Path((prompt_id, version_id)): Path<(i64, i64)>,
    State(state): State<AppState>,
) -> Result<Json<PromptResponse>, AppError> {
    // Set the active version in the database
    let prompt = state.db.prompt.set_active_prompt_version(prompt_id, version_id).await?;
    
    // Update the cache
    state.prompt_cache.insert(prompt_id, prompt.clone()).await;
    
    // Fetch associated tools
    let tools = state.db.tool.get_tools_by_prompt_version(prompt.version_id).await?;
    
    // Convert prompt to PromptResponse
    let mut response: PromptResponse = prompt.into();
    
    // Add tools to the response
    response.tools = tools.into_iter().map(|t| t.into()).collect();
    
    Ok(Json(response))
}

// OpenAI compatible API endpoints
#[axum::debug_handler]
pub async fn api_completions(
    State(state): State<AppState>,
    Json(payload): Json<ChatCompletionRequest>,
) -> Result<CompletionResponse, AppError> {
    if payload.messages.is_empty() {
        return Err(AppError::BadRequest(
            "Messages array cannot be empty".into(),
        ));
    }

    let prompt_key = &payload.model;
    let prompt = state
        .db
        .prompt
        .get_prompt_by_key(prompt_key)
        .await
        .map_err(|_| AppError::NotFound(format!("`Model` input with `Prompt Key` '{}' not found", prompt_key)))?;
    let json_mode = prompt.json_mode;

    // Fetch associated tools
    let tools_list = state.db.tool.get_tools_by_prompt_version(prompt.version_id).await?;
    let tools = tools_list.into_iter().map(|t| {
        ChatCompletionRequestTool::Function {
            function: ChatCompletionRequestFunctionDescription {
                name: t.tool_name,
                description: Some(t.description),
                parameters: serde_json::from_str(&t.parameters).unwrap_or_default(),
                strict: Some(t.strict)
            }
        }
    }).collect::<Vec<_>>();

    // Clone and modify the request to include prompt-associated tools,
    // but if streaming and tools are present, do NOT attach tools
    let mut payload = payload.clone();
    payload.tools = Some(tools);

    let is_stream = payload.stream.unwrap_or(false);

    // Insert into cache
    state.prompt_cache.insert(prompt.id, prompt.clone()).await;

    if is_stream {
        // Handle streaming request
        // Create LlmServiceRequest with streaming enabled
        payload.stream = Some(true);
        
        // Use our unified new() method
        let llm_props = LlmServiceRequest::new(prompt, payload)
            .map_err(|e| {
                tracing::error!("Error creating LlmServiceRequest: {}", e);
                AppError::InternalServerError("Failed to process request".into())
            })?;

        let (tx, mut rx) = mpsc::channel(100);
        let llm = Llm::new(llm_props, state.db.log);

        tokio::spawn(async move {
            let _ = llm.stream(tx).await;
        });

        let stream: SseStream = Box::pin(async_stream::stream! {
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
        });

        Ok(CompletionResponse::Stream(
            Sse::new(stream).keep_alive(KeepAlive::default())
        ))
    } else {
        // Handle non-streaming request
        // Create LlmServiceRequest with our new unified method
        let llm_props = LlmServiceRequest::new(prompt, payload)
            .map_err(|e| {
                tracing::error!("Error creating LlmServiceRequest: {}", e);
                AppError::InternalServerError("Failed to process request".into())
            })?;

        let llm = Llm::new(llm_props.clone(), state.db.log.clone());

        let res = if json_mode {
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

        Ok(CompletionResponse::Json(Json(res.0)))
    }
}

type SseStream = Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send>>;

pub enum CompletionResponse {
    Json(Json<LlmServiceChatCompletionResponse>),
    Stream(Sse<SseStream>),
}

impl IntoResponse for CompletionResponse {
    fn into_response(self) -> Response {
        match self {
            CompletionResponse::Json(json) => json.into_response(),
            CompletionResponse::Stream(sse) => sse.into_response(),
        }
    }
}
