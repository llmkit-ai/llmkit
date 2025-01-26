use serde_json::Value;

use crate::{common::types::models::ModelName, db::types::prompt::PromptWithModel};

pub struct LlmProps {
    pub model: ModelName,
    pub prompt: String,
    pub context: Value
}

impl LlmProps {
    pub fn from_prompt(prompt: PromptWithModel, context: Value) -> Self {
        let model_name: ModelName = prompt.model_name.into();
        LlmProps { 
            model: model_name, 
            prompt: prompt.prompt, 
            context
        }
    }
}
