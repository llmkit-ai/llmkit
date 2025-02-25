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
}


#[derive(Serialize, Clone)]
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
    
    pub fn for_chat(
        prompt: PromptRowWithModel, 
        context: serde_json::Value, 
        chat_messages: Vec<Message>
    ) -> Result<Self, LlmPropsError> {
        // For chat mode, we need to prepare a different message structure:
        // 1. System message comes from the prompt template
        // 2. First user message is either the rendered user template (if this is first message)
        //    or the first user message from chat_messages if continuing conversation
        // 3. Subsequent messages come from chat_messages

        // First, render the system prompt with context (always needed)
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
        
        let mut messages = Vec::new();
        
        // Always add the system message first
        messages.push(Message::System { content: rendered_system_prompt });
        
        // If chat_messages is empty, this is the first message, so use the template
        if chat_messages.is_empty() && prompt.prompt_type != "dynamic_both" {
            // Only render user template for first message
            tera.add_raw_template("user_prompt", &prompt.user)?;
            let rendered_user_prompt = tera.render("user_prompt", &tera_ctx)
                .map_err(|e| LlmPropsError::TeraRenderError(e))?;
            
            messages.push(Message::User { content: rendered_user_prompt });
        } else {
            // Otherwise, use the provided chat history
            // Skip system messages in chat_messages as we've already added our own
            for msg in chat_messages {
                if let Message::System { .. } = msg {
                    continue; // Skip system messages from chat history
                }
                messages.push(msg);
            }
        }

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
}
