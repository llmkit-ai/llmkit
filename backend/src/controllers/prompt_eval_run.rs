use axum::{
    extract::{Path, State},
    Json
};
use serde_json::Value;

use crate::{services::{llm::Llm, types::llm_props::LlmProps}, AppError, AppState};
use super::types::response::prompts::PromptExecutionResponse;


#[axum::debug_handler]
pub async fn execute_eval_run(
    Path((prompt_id, prompt_version_id)): Path<(i64, i64)>,
    State(state): State<AppState>
) -> Result<Json<PromptExecutionResponse>, AppError> {
    let prompt = state.db.prompt.get_prompt(prompt_id).await?;
    let evals = state.db.prompt_eval.get_by_prompt(prompt_id).await?;
    let mut eval_runs = vec![];

    for e in evals.iter() {
        let payload: Value = serde_json::from_str(&e.input_data)
            .map_err(|_| AppError::InternalServerError("Something went parsing input data".to_string()))?;

        let llm_props = LlmProps::new(prompt.clone(), payload).map_err(|e| {
            tracing::error!("{}", e);
            AppError::InternalServerError("An error occured processing prompt template".to_string())
        })?;

        let llm = Llm::new(llm_props, state.db.log.clone());
        let res = llm.text()
            .await
            .map_err(|_| AppError::InternalServerError("Something went wrong".to_string()))?;

        let eval_run = state.db.prompt_eval_run.create(prompt_version_id, e.id, None, &res.content).await?;
        eval_runs.push(eval_run);
    }


    todo!()
}
