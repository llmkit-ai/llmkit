use serde::{Deserialize, Serialize};

use crate::db::types::{log::LogRowModel, prompt::PromptWithModel};


// GET PROMPT RESPONSE
#[derive(Debug, Serialize)]
pub struct PromptResponse {
    pub id: i64,
    pub key: String,
    pub system: String,
    pub user: String,
    pub model: String,
    pub model_id: i64,
    pub provider: String,
    pub max_tokens: i64,
    pub temperature: f64,
    pub json_mode: bool
}


impl From<PromptWithModel> for PromptResponse {
    fn from(prompt: PromptWithModel) -> Self {
        PromptResponse {
            id: prompt.id,
            key: prompt.key,
            system: prompt.system,
            user: prompt.user,
            model: prompt.model_name.into(),
            model_id: prompt.model_id,
            provider: prompt.provider,
            max_tokens: prompt.max_tokens,
            temperature: prompt.temperature,
            json_mode: prompt.json_mode,
        }
    }
}



// PROMPT EXECUTION RESPONSE
#[derive(Debug, Serialize, Deserialize)]
pub struct PromptExecutionResponse {
    pub content: String,
    pub log: ApiLogResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiLogResponse {
    pub id: i64,
    pub prompt_id: Option<i64>,
    pub model_id: i64,
    pub status_code: Option<i64>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub reasoning_tokens: Option<i64>,
    pub request_body: Option<String>,
    pub response_data: Option<String>
}

impl From<LogRowModel> for ApiLogResponse {
    fn from(log: LogRowModel) -> Self {
        ApiLogResponse {
            id: log.id,
            prompt_id: log.prompt_id,
            model_id: log.model_id,
            status_code: log.status_code,
            input_tokens: log.input_tokens,
            output_tokens: log.output_tokens,
            reasoning_tokens: log.reasoning_tokens,
            request_body: log.request_body,
            response_data: log.response_data
        }
    }
}

impl PromptExecutionResponse {
    pub fn from_log_row(content: String, log_row: LogRowModel) -> Self {
        PromptExecutionResponse {
            content,
            log: ApiLogResponse::from(log_row),
        }
    }
}
