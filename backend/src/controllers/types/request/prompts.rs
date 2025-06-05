use serde::{Deserialize, Serialize};

use crate::common::types::chat_request::ChatCompletionRequestMessage;


#[derive(Debug, Deserialize)]
pub struct CreatePromptRequest {
    pub system: String,
    pub user: String,
    pub key: String,
    pub model_id: i64,
    pub max_tokens: i64,
    pub temperature: f64,
    pub json_mode: bool,
    pub json_schema: Option<String>,
    pub prompt_type: String,
    pub is_chat: bool,
    pub reasoning_effort: Option<String>
}

#[derive(Debug, Deserialize)]
pub struct UpdatePromptRequest {
    pub system: String,
    pub user: String,
    pub key: String,
    pub model_id: i64,
    pub max_tokens: i64,
    pub temperature: f64,
    pub json_mode: bool,
    pub json_schema: Option<String>,
    pub prompt_type: String,
    pub is_chat: bool,
    pub reasoning_effort: Option<String>
}

#[derive(Debug, Serialize)]
pub struct ChatExecuteRequest {
    /// Context for template variables (for the first message)
    #[serde(default)]
    pub context: serde_json::Value,
    
    /// Chat history including both user and assistant messages
    /// The first message should always be from the user
    pub messages: Vec<ChatCompletionRequestMessage>
}
