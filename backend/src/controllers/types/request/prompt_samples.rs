use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct CreatePromptSampleRequest {
    pub prompt_id: i64,
    pub input_data: Value,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePromptSampleRequest {
    pub prompt_id: i64,
    pub input_data: Value,
    pub name: String,
}
