use serde::Deserialize;



#[derive(Debug, Deserialize)]
pub struct CreatePromptRequest {
    pub prompt: String,
    pub model: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePromptRequest {
    pub prompt: String,
    pub model: String,
}
