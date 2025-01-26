use serde_json::Value;

use crate::{
    common::types::models::ModelName, 
    db::types::prompt::PromptWithModel
};

pub struct LlmProps {
    pub model: ModelName,
    pub max_tokens: i64,
    pub temperature: f64,
    pub json_mode: bool,
    pub prompt: String,
    pub context: Value,
}

impl LlmProps {
    pub fn from_prompt(prompt: PromptWithModel, context: Value) -> Self {
        let model_name: ModelName = prompt.model_name.into();

        LlmProps {
            model: model_name,
            max_tokens: prompt.max_tokens,
            temperature: prompt.temperature,
            json_mode: prompt.json_mode,
            prompt: prompt.prompt,
            context,
        }
    }
}
