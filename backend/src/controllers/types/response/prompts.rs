use serde::{Deserialize, Serialize};

use crate::db::types::{log::LogRowModel, prompt::PromptRowWithModel};


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
    pub json_mode: bool,
    pub version_number: i64,
    pub system_version_diff: Option<String>,
    pub user_version_diff: Option<String>,
    pub updated_at: String
}


impl From<PromptRowWithModel> for PromptResponse {
    fn from(prompt: PromptRowWithModel) -> Self {
        PromptResponse {
            id: prompt.id,
            key: prompt.key,
            system: prompt.system,
            user: prompt.user,
            model: prompt.model_name.into(),
            model_id: prompt.model_id,
            provider: prompt.provider_name.into(),
            max_tokens: prompt.max_tokens,
            temperature: prompt.temperature,
            json_mode: prompt.json_mode,
            version_number: prompt.version_number,
            system_version_diff: prompt.system_diff,
            user_version_diff: prompt.user_diff,
            updated_at: prompt.updated_at.to_string()
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
    pub model_name: String,
    pub response_data: Option<String>,
    pub status_code: Option<i64>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub request_body: Option<String>,
    pub created_at: Option<String>
}

impl From<LogRowModel> for ApiLogResponse {
    fn from(log: LogRowModel) -> Self {
        ApiLogResponse {
            id: log.id,
            prompt_id: log.prompt_id,
            model_id: log.model_id,
            model_name: log.model_name,
            status_code: log.status_code,
            input_tokens: log.input_tokens,
            output_tokens: log.output_tokens,
            request_body: log.request_body,
            response_data: log.response_data,
            created_at: log.created_at.map(|c| c.to_string())
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
