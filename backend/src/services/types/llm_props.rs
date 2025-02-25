use serde::Serialize;
use tera::{Context, Tera};

use crate::{
    common::types::models::LlmApiProvider, db::types::prompt::PromptRowWithModel
};

use super::message::Message;

#[derive(Debug, thiserror::Error)]
pub enum LlmPropsError {
    #[error("Tera templating error: {0}")]
    TeraTemplateError(#[from] tera::Error),
    #[error("Tera render error: {0}")]
    TeraRenderError(tera::Error),
    #[error("Malformed input for Chat")]
    ChatMessagesInputError,
}


#[derive(Serialize, Clone, Debug)]
pub struct LlmProps {
    pub provider: LlmApiProvider,
    pub model_name: String,
    pub max_tokens: i64,
    pub temperature: f64,
    pub json_mode: bool,
    pub messages: Vec<Message>,
    pub prompt_id: i64,
    pub model_id: i64
}

impl LlmProps {
    pub fn new(prompt: PromptRowWithModel, context: serde_json::Value) -> Result<Self, LlmPropsError> {
        let mut tera = Tera::default();
        tera.add_raw_template("system_prompt", &prompt.system)?;
        tera.add_raw_template("user_prompt", &prompt.user)?;

        let mut tera_ctx = Context::new();
        if let serde_json::Value::Object(context) = context {
            for (k, v) in context {
                tera_ctx.insert(k, &v);
            }
        }

        let rendered_system_prompt = tera.render("system_prompt", &tera_ctx)
            .map_err(|e| LlmPropsError::TeraRenderError(e))?;
        let rendered_user_prompt = tera.render("user_prompt", &tera_ctx)
            .map_err(|e| LlmPropsError::TeraRenderError(e))?;

        let messages = vec![ Message::System { content: rendered_system_prompt }, Message::User { content: rendered_user_prompt } ];

        Ok(LlmProps {
            provider: prompt.provider_name.into(),
            model_name: prompt.model_name,
            max_tokens: prompt.max_tokens,
            temperature: prompt.temperature,
            json_mode: prompt.json_mode,
            messages,
            prompt_id: prompt.id,
            model_id: prompt.model_id,
        })
    }

    pub fn new_split_context(prompt: PromptRowWithModel, system_context: serde_json::Value, user_context: serde_json::Value) -> Result<Self, LlmPropsError> {
        let mut tera = Tera::default();
        tera.add_raw_template("system_prompt", &prompt.system)?;
        tera.add_raw_template("user_prompt", &prompt.user)?;

        let mut system_ctx = Context::new();
        if let serde_json::Value::Object(context) = system_context {
            for (k, v) in context {
                system_ctx.insert(k, &v);
            }
        }

        let mut user_ctx = Context::new();
        if let serde_json::Value::Object(context) = user_context {
            for (k, v) in context {
                user_ctx.insert(k, &v);
            }
        }

        let rendered_system_prompt = tera.render("system_prompt", &system_ctx)
            .map_err(|e| LlmPropsError::TeraRenderError(e))?;
        let rendered_user_prompt = tera.render("user_prompt", &user_ctx)
            .map_err(|e| LlmPropsError::TeraRenderError(e))?;

        let messages = vec![ Message::System { content: rendered_system_prompt }, Message::User { content: rendered_user_prompt } ];

        Ok(LlmProps {
            provider: prompt.provider_name.into(),
            model_name: prompt.model_name,
            max_tokens: prompt.max_tokens,
            temperature: prompt.temperature,
            json_mode: prompt.json_mode,
            messages,
            prompt_id: prompt.id,
            model_id: prompt.model_id,
        })
    }

    pub fn new_chat(
        prompt: PromptRowWithModel, 
        context: serde_json::Value, 
        messages: Vec<Message>
    ) -> Result<Self, LlmPropsError> {
        // Always render the system prompt with context
        let mut tera = Tera::default();
        tera.add_raw_template("system_prompt", &prompt.system)?;
        
        let mut tera_ctx = Context::new();
        if let serde_json::Value::Object(context) = context {
            for (k, v) in context {
                tera_ctx.insert(k, &v);
            }
        }

        let rendered_system_prompt = tera.render("system_prompt", &tera_ctx)
            .map_err(|e| LlmPropsError::TeraRenderError(e))?;
        
        let mut messages = messages;
        
        // Check if there's already a system message in the input
        let has_system_message = messages.iter().any(|msg| matches!(msg, Message::System { .. }));
        
        if has_system_message {
            // Replace the first system message with our rendered one
            let system_index = messages.iter().position(|msg| matches!(msg, Message::System { .. })).unwrap();
            messages[system_index] = Message::System { content: rendered_system_prompt };
        } else {
            // No system message found, add one at the beginning
            messages.insert(0, Message::System { content: rendered_system_prompt });
        }

        Ok(LlmProps {
            provider: prompt.provider_name.into(),
            model_name: prompt.model_name,
            max_tokens: prompt.max_tokens,
            temperature: prompt.temperature,
            json_mode: prompt.json_mode,
            messages: messages.to_vec(),
            prompt_id: prompt.id,
            model_id: prompt.model_id,
        })
    }
}
