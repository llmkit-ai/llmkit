use serde::{Deserialize, Serialize};
use crate::services::types::message::Message;


#[derive(Debug, Deserialize)]
pub struct CreatePromptRequest {
    pub system: String,
    pub user: String,
    pub key: String,
    pub model_id: i64,
    pub max_tokens: i64,
    pub temperature: f64,
    pub json_mode: bool,
    pub prompt_type: String,
    pub is_chat: bool
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
    pub prompt_type: String,
    pub is_chat: bool
}

#[derive(Debug, Deserialize)]
pub struct ChatExecuteRequest {
    /// Context for template variables (for the first message)
    #[serde(default)]
    pub context: serde_json::Value,
    
    /// Chat history including both user and assistant messages
    /// The first message should always be from the user
    pub messages: Vec<Message>
}

/// API request following OpenAI standard
#[derive(Debug, Deserialize, Serialize)]
pub struct ApiCompletionRequest {
    /// The prompt key to use (maps to model in OpenAI spec)
    pub model: String,
    
    /// For chat completions - array of messages (includes system, user, assistant)
    /// Note: In our implementation, for the first message:
    /// - If system message exists, its content is treated as JSON context for our template
    /// - If user message exists, for multi-turn conversations it's used directly, 
    ///   but for templating it may be ignored if we're using our stored template
    pub messages: Vec<Message>,
    
    /// Maximum number of tokens to generate
    #[serde(default)]
    pub max_tokens: Option<i64>,
    
    /// What sampling temperature to use, between 0 and 2
    #[serde(default)]
    pub temperature: Option<f64>,
    
    /// Whether to stream the response
    #[serde(default)]
    pub stream: Option<bool>,
    
    /// An object specifying the format that the model must output
    #[serde(default)]
    pub response_format: Option<ResponseFormat>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResponseFormat {
    /// Currently only supports "json_object" as per OpenAI spec
    #[serde(default)]
    pub r#type: String
}
