use serde::Serialize;
use tera::{Context, Tera};
use serde_json::Value;

use crate::{
    common::types::{
        message::{ChatCompletionRequest, ChatCompletionRequestMessage}, 
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
    pub prompt_id: i64,
    pub model_id: i64,
    pub request: ChatCompletionRequest
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
            let user_content = request.messages.iter()
                .find(|msg| msg.is_user())
                .map(|msg| msg.content().to_string())
                .unwrap_or("".to_string());

            // Simple mode - system + user message with template
            // For dynamic_both, we need to extract user context separately
            if prompt.prompt_type == "dynamic_both" {
                let user_context = serde_json::from_str::<Value>(&user_content)
                    .map_err(|_| LlmServiceRequestError::ChatMessagesInputError)?;
                
                let mut user_ctx = Context::new();
                if let Value::Object(context) = user_context {
                    for (k, v) in context {
                        user_ctx.insert(k, &v);
                    }
                }
                
                let rendered_user_prompt = tera.render("user_prompt", &user_ctx)
                    .map_err(|e| LlmServiceRequestError::TeraRenderError(e))?;
                
                vec![
                    ChatCompletionRequestMessage::System { content: rendered_system_prompt, name: None },
                    ChatCompletionRequestMessage::User { content: rendered_user_prompt, name: None }
                ]
            } else {
                vec![
                    ChatCompletionRequestMessage::System { content: rendered_system_prompt, name: None },
                    ChatCompletionRequestMessage::User { content: user_content, name: None }
                ]
            }
        };

        // Create request with all properties and overrides
        let mut service_request = LlmServiceRequest {
            prompt_id: prompt.id,
            model_id: prompt.model_id,
            provider: prompt.provider_name.clone().into(),
            base_url: prompt.provider_base_url.clone(),
            request: request.clone() 
        };
        
        // Override input with inputs from Prompt table
        service_request.request.max_tokens = Some(prompt.max_tokens);
        service_request.request.model = prompt.model_name;
        service_request.request.temperature = Some(prompt.temperature);
        if prompt.json_mode {
            service_request.request.response_format = Some("{\"type\": \"json_object\"}".to_string());
        }
        
        Ok(service_request)
    }
}
