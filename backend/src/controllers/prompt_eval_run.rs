use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use super::types::{
    request::prompt_eval_run::UpdateEvalRunRequest,
    response::prompt_eval_run::{
        PromptEvalExecutionRunResponse, PromptEvalRunResponse, PromptEvalVersionPerformanceResponse,
    },
};

use crate::{
    common::types::chat_request::ChatCompletionRequest,
    services::{llm::Llm, types::llm_service::LlmServiceRequest},
    AppError, AppState,
};

pub async fn execute_eval_run(
    Path((prompt_id, prompt_version_id)): Path<(i64, i64)>,
    State(state): State<AppState>,
) -> Result<Json<PromptEvalExecutionRunResponse>, AppError> {
    let prompt = state.db.prompt.get_prompt(prompt_id).await?;
    let evals = state.db.prompt_eval.get_by_prompt(prompt_id).await?;
    let run_id = Uuid::new_v4().to_string();
    let mut eval_runs = vec![];

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
                // System message with context
                crate::common::types::chat_request::ChatCompletionRequestMessage::System {
                    content: system_content,
                    name: None,
                },
                // User message with content
                crate::common::types::chat_request::ChatCompletionRequestMessage::User {
                    content: user_content,
                    name: None,
                },
            ],
            stream: None,
            response_format: None,
            tools: None,
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
        let res = llm
            .text()
            .await
            .map_err(|_| AppError::InternalServerError("Something went wrong".to_string()))?;

        if let Some(c) = res.0.choices.first() {
            // TODO: We should make the DB field nullable so we don't have to hack this
            let content = c.message.content.clone().map(|c| c.to_string()).unwrap_or("".to_string());

            let eval_run = state
                .db
                .prompt_eval_run
                .create(&run_id, prompt_version_id, e.id, None, &content)
                .await?;

            eval_runs.push(eval_run);
        }
    }

    Ok(Json(eval_runs.into()))
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
