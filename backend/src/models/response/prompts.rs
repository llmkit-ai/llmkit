use serde::Serialize;

use crate::db::prompts::LlmPrompt;


#[derive(Debug, Serialize)]
pub struct PromptResponse {
    pub id: i64,
    pub key: String,
    pub prompt: String,
    pub model: String
}


impl From<LlmPrompt> for PromptResponse {
    fn from(prompt: LlmPrompt) -> Self {
        PromptResponse {
            id: prompt.id,
            key: prompt.key,
            prompt: prompt.prompt,
            model: prompt.model,
        }
    }
}

