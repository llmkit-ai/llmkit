use serde::Serialize;
use tera::{Context, Tera};

use crate::{
    common::types::{message::ChatCompletionRequest, models::LlmApiProvider}, 
    db::types::prompt::PromptRowWithModel
};


#[derive(Debug, thiserror::Error)]
pub enum LlmServiceRequestError {
    #[error("Tera templating error: {0}")]
    TeraTemplateError(#[from] tera::Error),
    #[error("Tera render error: {0}")]
    TeraRenderError(tera::Error),
    #[error("Malformed input for Chat")]
    ChatMessagesInputError,
}


#[derive(Serialize, Clone, Debug)]
pub struct LlmServiceRequest {
    pub provider: LlmApiProvider,
    pub base_url: String,
    pub chat_request: ChatCompletionRequest
}

impl LlmServiceRequest {
    pub fn new(prompt: PromptRowWithModel, context: serde_json::Value) -> Result<Self, LlmServiceRequestError> {
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
            .map_err(|e| LlmServiceRequestError::TeraRenderError(e))?;
        let rendered_user_prompt = tera.render("user_prompt", &tera_ctx)
            .map_err(|e| LlmServiceRequestError::TeraRenderError(e))?;

        let messages = vec![ Message::System { content: rendered_system_prompt }, Message::User { content: rendered_user_prompt } ];

        Ok(LlmServiceRequest {
            provider: prompt.provider_name.into(),
            base_url: prompt.provider_base_url,
            model_name: prompt.model_name,
            max_tokens: prompt.max_tokens,
            temperature: prompt.temperature,
            json_mode: prompt.json_mode,
            messages,
            prompt_id: prompt.id,
            model_id: prompt.model_id,
            stream: None,
            tools: None,
            fallback_models: None,
            transforms: None,
        })
    }

    pub fn new_split_context(prompt: PromptRowWithModel, system_context: serde_json::Value, user_context: serde_json::Value) -> Result<Self, LlmServiceRequestError> {
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
            .map_err(|e| LlmServiceRequestError::TeraRenderError(e))?;
        let rendered_user_prompt = tera.render("user_prompt", &user_ctx)
            .map_err(|e| LlmServiceRequestError::TeraRenderError(e))?;

        let messages = vec![ Message::System { content: rendered_system_prompt }, Message::User { content: rendered_user_prompt } ];

        Ok(LlmServiceRequest {
            provider: prompt.provider_name.into(),
            base_url: prompt.provider_base_url,
            model_name: prompt.model_name,
            max_tokens: prompt.max_tokens,
            temperature: prompt.temperature,
            json_mode: prompt.json_mode,
            messages,
            prompt_id: prompt.id,
            model_id: prompt.model_id,
            stream: None,
            tools: None,
            fallback_models: None,
            transforms: None,
        })
    }

    pub fn new_chat(
        prompt: PromptRowWithModel, 
        context: serde_json::Value, 
        messages: Vec<Message>
    ) -> Result<Self, LlmServiceRequestError> {
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
            .map_err(|e| LlmServiceRequestError::TeraRenderError(e))?;
        
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

        Ok(LlmServiceRequest {
            provider: prompt.provider_name.into(),
            base_url: prompt.provider_base_url,
            model_name: prompt.model_name,
            max_tokens: prompt.max_tokens,
            temperature: prompt.temperature,
            json_mode: prompt.json_mode,
            messages: messages.to_vec(),
            prompt_id: prompt.id,
            model_id: prompt.model_id,
            stream: None,
            tools: None,
            fallback_models: None,
            transforms: None,
        })
    }
    
    // Function to enable streaming
    pub fn with_stream(mut self, enable: bool) -> Self {
        self.stream = Some(enable);
        self
    }
    
    // Function to add tools
    pub fn with_tools(mut self, tools: Vec<LlmServiceRequestTool>) -> Self {
        self.tools = Some(tools);
        self
    }
    
    // Function to add fallback models
    pub fn with_fallback_models(mut self, models: Vec<String>) -> Self {
        self.fallback_models = Some(models);
        self
    }
    
    // Function to add transforms
    pub fn with_transforms(mut self, transforms: Vec<String>) -> Self {
        self.transforms = Some(transforms);
        self
    }
}
