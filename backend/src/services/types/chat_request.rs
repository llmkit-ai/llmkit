use openrouter_api::models::tool::{FunctionCall, ToolCall};
use serde::{Deserialize, Serialize};
use tera::{Context, Tera};

use crate::{
    common::types::models::LlmApiProvider, 
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
pub struct LlmServiceRequestTool {
    #[serde(rename = "type")]
    pub kind: String,
    pub function: LlmServiceRequestToolFunction,
}

#[derive(Serialize, Clone, Debug)]
pub struct LlmServiceRequestToolFunction {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct LlmServiceRequestToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(rename = "function")]
    pub function_call: LlmServiceRequestFunctionCall,
}

// Impl for Opernrouter SDK
impl Into<ToolCall> for LlmServiceRequestToolCall {
    fn into(self) -> ToolCall {
        ToolCall { id: self.id, kind: self.kind, function_call: self.function_call.into() }
    }
}


#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct LlmServiceRequestFunctionCall {
    pub name: String,
    pub arguments: String,
}

// Impl for Opernrouter SDK
impl Into<FunctionCall> for LlmServiceRequestFunctionCall {
    fn into(self) -> FunctionCall {
        FunctionCall {
            name: self.name,
            arguments: self.arguments
        }
    }
}


#[derive(Serialize, Clone, Debug)]
pub struct LlmServiceRequest {
    pub provider: LlmApiProvider,
    pub base_url: String,
    pub model_name: String,
    pub max_tokens: i64,
    pub temperature: f64,
    pub json_mode: bool,
    pub messages: Vec<Message>,
    pub prompt_id: i64,
    pub model_id: i64,
    // New fields from ChatCompletionRequest
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<LlmServiceRequestTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_models: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transforms: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase", tag = "role")]
pub enum Message {
    System { content: String },
    User { content: String },
    #[serde(rename_all = "camelCase")]
    Assistant { 
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_calls: Option<Vec<LlmServiceRequestToolCall>> 
    },
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
