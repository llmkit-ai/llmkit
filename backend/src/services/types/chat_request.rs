use serde::Serialize;
use tera::{Context, Tera};
use serde_json::Value;

use crate::{
    common::types::{
        message::{ChatCompletionRequest, ChatCompletionRequestMessage, ChatCompletionRequestTool}, 
        models::LlmApiProvider
    }, 
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
    pub model_name: String,
    pub max_tokens: i64,
    pub temperature: f64,
    pub json_mode: bool,
    pub messages: Vec<ChatCompletionRequestMessage>,
    pub prompt_id: i64,
    pub model_id: i64,
    pub stream: Option<bool>,
    pub tools: Option<Vec<ChatCompletionRequestTool>>,
    pub fallback_models: Option<Vec<String>>,
    pub transforms: Option<Vec<String>>,
}

impl LlmServiceRequest {
    pub fn new(prompt: PromptRowWithModel, request: ChatCompletionRequest) -> Result<Self, LlmServiceRequestError> {
        let mut tera = Tera::default();
        tera.add_raw_template("system_prompt", &prompt.system)?;
        tera.add_raw_template("user_prompt", &prompt.user)?;
        
        // Always extract context from system message if available
        let system_context = request.messages.iter()
            .find(|msg| msg.is_system())
            .and_then(|msg| serde_json::from_str::<Value>(&msg.content()).ok())
            .unwrap_or(Value::Object(serde_json::Map::new()));
        
        // Render system prompt with context
        let mut system_ctx = Context::new();
        if let Value::Object(context) = system_context {
            for (k, v) in context {
                system_ctx.insert(k, &v);
            }
        }
        
        let rendered_system_prompt = tera.render("system_prompt", &system_ctx)
            .map_err(|e| LlmServiceRequestError::TeraRenderError(e))?;

        // Determine messages based on request
        let messages = if request.messages.len() > 2 {
            // Chat mode - keep existing messages but replace/insert system message
            let mut new_messages = request.messages.clone();
            
            // Replace or add system message
            if let Some(pos) = new_messages.iter().position(|msg| msg.is_system()) {
                new_messages[pos] = ChatCompletionRequestMessage::System { 
                    content: rendered_system_prompt,
                    name: None 
                };
            } else {
                new_messages.insert(0, ChatCompletionRequestMessage::System { 
                    content: rendered_system_prompt,
                    name: None 
                });
            }
            
            new_messages
        } else {
            // Simple mode - system + user message with template
            // For dynamic_both, we need to extract user context separately
            let user_ctx = if prompt.prompt_type == "dynamic_both" {
                let user_context = request.messages.iter()
                    .find(|msg| msg.is_user())
                    .and_then(|msg| serde_json::from_str::<Value>(&msg.content()).ok())
                    .unwrap_or(Value::Object(serde_json::Map::new()));
                
                let mut ctx = Context::new();
                if let Value::Object(context) = user_context {
                    for (k, v) in context {
                        ctx.insert(k, &v);
                    }
                }
                ctx
            } else {
                // For other types, use the same context as system
                system_ctx.clone()
            };
            
            let rendered_user_prompt = tera.render("user_prompt", &user_ctx)
                .map_err(|e| LlmServiceRequestError::TeraRenderError(e))?;
            
            vec![
                ChatCompletionRequestMessage::System { content: rendered_system_prompt, name: None },
                ChatCompletionRequestMessage::User { content: rendered_user_prompt, name: None }
            ]
        };
        
        // Create request with all properties and overrides
        let mut service_request = LlmServiceRequest {
            provider: prompt.provider_name.clone().into(),
            base_url: prompt.provider_base_url.clone(),
            model_name: prompt.model_name.clone(),
            max_tokens: prompt.max_tokens,
            temperature: prompt.temperature,
            json_mode: prompt.json_mode, 
            messages,
            prompt_id: prompt.id,
            model_id: prompt.model_id,
            stream: request.stream,
            tools: request.tools.clone(),
            fallback_models: request.models.clone(),
            transforms: request.transforms.clone(),
        };
        
        // Apply request overrides if specified
        if let Some(max_tokens) = request.max_tokens {
            service_request.max_tokens = max_tokens;
        }
        
        if let Some(temperature) = request.temperature {
            service_request.temperature = temperature;
        }
        
        // Set JSON mode if specified in request format
        if let Some(ref response_format) = request.response_format {
            if response_format == "json_object" {
                service_request.json_mode = true;
            }
        }
        
        Ok(service_request)
    }

    // Function to enable streaming
    pub fn with_stream(mut self, enable: bool) -> Self {
        self.stream = Some(enable);
        self
    }
    
    // Function to add tools
    pub fn with_tools(mut self, tools: Vec<ChatCompletionRequestTool>) -> Self {
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
