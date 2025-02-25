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
        // For chat input we need to
        // 1. Determine if this is the first input based on messages length
        // 2. If it is, then and only then do the templating
        // 3. If not, simply return the messages as is

        let messages_len = messages.len();

        if messages_len > 2 {
            return Ok(LlmProps {
                provider: prompt.provider_name.into(),
                model_name: prompt.model_name,
                max_tokens: prompt.max_tokens,
                temperature: prompt.temperature,
                json_mode: prompt.json_mode,
                messages: messages.to_vec(),
                prompt_id: prompt.id,
                model_id: prompt.model_id,
            });
        }

        let mut messages = messages;

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
        
        // If there are two messages, this indicates that the user passed in both a
        // system and user message and in this case we should just swap out the system message.
        //
        // If there is only 1 message, then we know that is just the User message because
        // the system prompt may have not needed to be dynamic at all, and thus had no context.
        // In this case we have to insert the system message before the user message
        if messages_len == 2 {
            messages[0] = Message::System { content: rendered_system_prompt };
        } else {
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
