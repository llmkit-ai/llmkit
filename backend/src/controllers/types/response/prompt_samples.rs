use serde::Serialize;
use crate::db::types::prompt_sample::PromptSample;


#[derive(Debug, Serialize)]
pub struct PromptSampleResponse {
    pub id: i64,
    pub prompt_id: i64,
    pub input_data: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<PromptSample> for PromptSampleResponse {
    fn from(prompt: PromptSample) -> Self {
        PromptSampleResponse {
            id: prompt.id,
            prompt_id: prompt.prompt_id,
            input_data: prompt.input_data,
            name: prompt.name,
            created_at: prompt.created_at.to_string(),
            updated_at: prompt.updated_at.to_string()
        }
    }
}
