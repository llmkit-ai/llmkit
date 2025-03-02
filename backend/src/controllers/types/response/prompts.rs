use serde::{Deserialize, Serialize};
use chrono::Utc;

use crate::{db::types::{log::LogRowModel, prompt::PromptRowWithModel}, services::types::chat_request::Message};


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
    pub json_schema: Option<String>,
    pub prompt_type: String,
    pub is_chat: bool,
    pub version_id: i64,
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
            json_schema: prompt.json_schema,
            prompt_type: prompt.prompt_type,
            is_chat: prompt.is_chat,
            version_id: prompt.version_id,
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

// New OpenAI standard API responses

#[derive(Debug, Serialize)]
pub struct ApiCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<ApiChoice>,
    pub usage: ApiUsage,
}

#[derive(Debug, Serialize)]
pub struct ApiChoice {
    pub index: i64,
    pub message: Message,
    pub finish_reason: String,
}

#[derive(Debug, Serialize)]
pub struct ApiUsage {
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
}

// For streaming responses
#[derive(Debug, Serialize)]
pub struct ApiCompletionChunk {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<ApiChunkChoice>,
}

#[derive(Debug, Serialize)]
pub struct ApiChunkChoice {
    pub index: i64,
    pub delta: ApiDelta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApiDelta {
    pub content: Option<String>,
    pub role: Option<String>,
}

impl PromptExecutionResponse {
    pub fn to_api_response(&self, prompt_key: &str) -> ApiCompletionResponse {
        let now = Utc::now().timestamp();
        
        ApiCompletionResponse {
            id: format!("chatcmpl-{}", self.log.id),
            object: "chat.completion".to_string(),
            created: self.log.created_at.as_ref()
                .and_then(|date_str| date_str.parse::<chrono::DateTime<chrono::Utc>>().ok())
                .map(|dt| dt.timestamp())
                .unwrap_or(now),
            model: prompt_key.to_string(),
            choices: vec![
                ApiChoice {
                    index: 0,
                    message: Message::Assistant { content: self.content.clone(), tool_calls: None },
                    finish_reason: "stop".to_string(),
                }
            ],
            usage: ApiUsage {
                prompt_tokens: self.log.input_tokens.unwrap_or(0),
                completion_tokens: self.log.output_tokens.unwrap_or(0),
                total_tokens: self.log.input_tokens.unwrap_or(0) + self.log.output_tokens.unwrap_or(0),
            },
        }
    }
}
