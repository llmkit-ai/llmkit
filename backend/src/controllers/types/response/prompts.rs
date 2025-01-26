use serde::Serialize;

use crate::db::types::prompt::PromptWithModel;


#[derive(Debug, Serialize)]
pub struct PromptResponse {
    pub id: i64,
    pub key: String,
    pub prompt: String,
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
            prompt: prompt.prompt,
            model: prompt.model_name.into(),
            model_id: prompt.model_id,
            provider: prompt.provider,
            max_tokens: prompt.max_tokens,
            temperature: prompt.temperature,
            json_mode: prompt.json_mode,
        }
    }
}

