use serde::Serialize;

use crate::{
    common::types::models::LlmModel, 
    db::types::prompt::PromptWithModel
};

use super::message::Message;


#[derive(Serialize)]
pub struct LlmProps {
    pub model: LlmModel,
    pub max_tokens: i64,
    pub temperature: f64,
    pub json_mode: bool,
    pub messages: Vec<Message>,
    pub streaming: bool
}

impl LlmProps {
    pub fn new(prompt: PromptWithModel, messages: Vec<Message>, streaming: bool) -> Self {
        let model_name: LlmModel = prompt.model_name.into();

        LlmProps {
            model: model_name,
            max_tokens: prompt.max_tokens,
            temperature: prompt.temperature,
            json_mode: prompt.json_mode,
            messages,
            streaming
        }
    }
}
