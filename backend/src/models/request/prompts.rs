use serde::Deserialize;



#[derive(Debug, Deserialize)]
pub struct CreatePromptRequest {
    pub prompt: String,
    pub key: String,
    pub model_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePromptRequest {
    pub prompt: String,
    pub key: String,
    pub model_id: i64,
}
