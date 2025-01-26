use serde::Deserialize;


#[derive(Debug, Deserialize)]
pub struct CreatePromptRequest {
    pub prompt: String,
    pub key: String,
    pub model_id: i64,
    pub max_tokens: i64,
    pub temperature: f64,
    pub json_mode: bool
}

#[derive(Debug, Deserialize)]
pub struct UpdatePromptRequest {
    pub prompt: String,
    pub key: String,
    pub model_id: i64,
    pub max_tokens: i64,
    pub temperature: f64,
    pub json_mode: bool
}
