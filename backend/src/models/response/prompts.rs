use serde::Serialize;

use crate::db::prompts::LlmPromptWithModel;


#[derive(Debug, Serialize)]
pub struct PromptResponse {
    pub id: i64,
    pub key: String,
    pub prompt: String,
    pub model: String,
    pub model_id: i64,
    pub provider: String
}


impl From<LlmPromptWithModel> for PromptResponse {
    fn from(prompt: LlmPromptWithModel) -> Self {
        PromptResponse {
            id: prompt.id,
            key: prompt.key,
            prompt: prompt.prompt,
            model: prompt.model_name,
            model_id: prompt.model_id,
            provider: prompt.provider,
        }
    }
}

