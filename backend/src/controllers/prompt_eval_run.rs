use axum::{
    extract::{Path, State, Query},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct EvalRunParams {
    pub rounds: Option<i64>,
}

use super::types::{
    request::prompt_eval_run::UpdateEvalRunRequest,
    response::prompt_eval_run::{
        PromptEvalExecutionRunResponse, PromptEvalRunResponse, PromptEvalVersionPerformanceResponse,
    },
};

use crate::{
    common::types::chat_request::{
        ChatCompletionRequest, ChatCompletionRequestTool, ChatCompletionRequestFunctionDescription
    },
    services::{llm::Llm, types::llm_service::LlmServiceRequest},
    AppError, AppState,
};

pub async fn execute_eval_run(
    Path((prompt_id, prompt_version_id)): Path<(i64, i64)>,
    State(state): State<AppState>,
    Query(params): Query<EvalRunParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let prompt = state.db.prompt.get_prompt(prompt_id).await?;
    let evals = state.db.prompt_eval.get_by_prompt(prompt_id).await?;
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

    let rounds = params.rounds.unwrap_or(1);
    let mut all_runs: Vec<PromptEvalExecutionRunResponse> = Vec::new();

    for e in evals.iter() {
        // Parse system_prompt_input if present
        let system_content = match &e.system_prompt_input {
            Some(system_json_str) => system_json_str.clone(),
            None => "{}".to_string(), // Empty object if no system input
        };

        // Always use user_prompt_input
        let user_content = e.user_prompt_input.clone();

        // Create a ChatCompletionRequest with the inputs
        let chat_request = ChatCompletionRequest {
            model: prompt.key.clone(),
            messages: vec![
                crate::common::types::chat_request::ChatCompletionRequestMessage::System {
                    content: system_content,
                    name: None,
                },
                crate::common::types::chat_request::ChatCompletionRequestMessage::User {
                    content: user_content,
                    name: None,
                },
            ],
            stream: None,
            response_format: None,
            tools: Some(tools.clone()),
            provider: None,
            models: None,
            transforms: None,
            max_tokens: Some(prompt.max_tokens as u32),
            temperature: Some(prompt.temperature as f32),
        };

        let llm_props = LlmServiceRequest::new(prompt.clone(), chat_request).map_err(|e| {
            tracing::error!("{}", e);
            AppError::InternalServerError("An error occured processing prompt template".to_string())
        })?;

        let llm = Llm::new(llm_props, state.db.log.clone());
        let mut eval_runs = Vec::new();

        for _ in 0..rounds {
            let run_id = Uuid::new_v4().to_string();

            let res = llm
                .text()
                .await
                .map_err(|e| {
                    tracing::error!("LLM service error: {}", e);
                    AppError::InternalServerError(format!("LLM service error: {}", e))
                })?;

            if let Some(c) = res.0.choices.first() {
                if let Some(content) = &c.message.content {
                    let eval_run = state
                        .db
                        .prompt_eval_run
                        .create(&run_id, prompt_version_id, e.id, None, &content)
                        .await?;

                    eval_runs.push(eval_run);
                }

                // for not just stringify the tool calls
                if let Some(tool_calls) = &c.message.tool_calls { 
                    let tool_calls_string = serde_json::to_string(&tool_calls)
                        .map_err(|e| AppError::InternalServerError(format!("Failed to serialize tool calls: {}", e)))?;

                    let eval_run = state
                        .db
                        .prompt_eval_run
                        .create(&run_id, prompt_version_id, e.id, None, &tool_calls_string)
                        .await?;

                    eval_runs.push(eval_run);
                }
            }

        }

        all_runs.push(eval_runs.into());
    }

    // Maintain backward compatibility: return single response when rounds=1
    if rounds == 1 && !all_runs.is_empty() {
        // Return single response for backward compatibility
        Ok(Json(serde_json::to_value(all_runs.into_iter().next()
            .ok_or_else(|| AppError::InternalServerError("No eval runs generated".to_string()))?)
            .map_err(|e| AppError::InternalServerError(format!("Failed to serialize response: {}", e)))?)
        )
    } else {
        // Return array of responses for multiple rounds
        Ok(Json(serde_json::to_value(all_runs)
            .map_err(|e| AppError::InternalServerError(format!("Failed to serialize response: {}", e)))?)
        )
    }
}

pub async fn get_eval_run_by_id(
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> Result<Json<PromptEvalRunResponse>, AppError> {
    let eval_run = state.db.prompt_eval_run.get_by_id(id).await?;
    Ok(Json(eval_run.into()))
}

pub async fn get_eval_performance_by_prompt_id(
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> Result<Json<Vec<PromptEvalVersionPerformanceResponse>>, AppError> {
    let performance = state
        .db
        .prompt_eval_run
        .get_prompt_version_performance(id)
        .await?;

    tracing::info!("performance: {:?}", performance);

    Ok(Json(performance.into_iter().map(|p| p.into()).collect()))
}

pub async fn get_eval_runs_by_prompt_version(
    Path((_prompt_id, prompt_version_id)): Path<(i64, i64)>,
    State(state): State<AppState>,
) -> Result<Json<Vec<PromptEvalRunResponse>>, AppError> {
    let eval_runs = state
        .db
        .prompt_eval_run
        .get_by_prompt_version(prompt_version_id)
        .await?;
    Ok(Json(eval_runs.into_iter().map(|run| run.into()).collect()))
}

pub async fn update_eval_run_score(
    Path(id): Path<i64>,
    State(state): State<AppState>,
    Json(request): Json<UpdateEvalRunRequest>,
) -> Result<Json<PromptEvalRunResponse>, AppError> {
    let updated_eval_run = state
        .db
        .prompt_eval_run
        .update_score(id, request.score)
        .await?;

    Ok(Json(updated_eval_run.into()))
}
