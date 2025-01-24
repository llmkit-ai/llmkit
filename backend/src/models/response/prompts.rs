use serde::Serialize;

use crate::db::models::prompt::PromptWithModel;



#[derive(Debug, Serialize)]
pub struct PromptResponse {
    pub id: i64,
    pub key: String,
    pub prompt: String,
    pub model: String,
    pub model_id: i64,
    pub provider: String
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
        }
    }
}

