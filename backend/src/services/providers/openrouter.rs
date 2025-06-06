use crate::common::types::chat_response::{LlmServiceChatCompletionChunk, LlmServiceChatCompletionResponse};
use crate::common::types::chat_request::ChatCompletionRequestMessage;
use crate::services::types::{
    llm_error::LlmError, llm_error::LlmStreamingError, llm_service::LlmServiceRequest
};
use anyhow::Result;
use futures_util::StreamExt;
use openrouter_api::models::tool::Tool;
use openrouter_api::{OpenRouterClient, Ready};
use openrouter_api::types::chat::ChatCompletionRequest;
use tokio::sync::mpsc::Sender;

use std::time::{SystemTime, UNIX_EPOCH};


pub struct OpenrouterProvider<'a> {
    props: &'a LlmServiceRequest,
    client: OpenRouterClient<Ready>,
}

impl<'a> OpenrouterProvider<'a> {
    /// Creates a new instance of `OpenrouterProvider` with the given properties and streaming flag.
    pub fn new(props: &'a LlmServiceRequest) -> Result<Self, LlmError> {
        let api_key = std::env::var("OPENROUTER_API_KEY")
            .map_err(|_| LlmError::InvalidConfig("Missing OPENROUTER_API_KEY".to_string()))?;

        let client = OpenRouterClient::new()
            .with_base_url("https://openrouter.ai/api/v1/")?
            .with_api_key(api_key)?;

        Ok(OpenrouterProvider {
            props,
            client,
        })
    }

    /// Builds an HTTP request using the OpenRouter API library's client configuration.
    pub async fn execute_chat(&self) -> Result<LlmServiceChatCompletionResponse, LlmError> {
        let messages = self.props.request.messages.iter().map(|msg| {
            openrouter_api::types::chat::Message {
                role: msg.role().to_string(),
                content: msg.content(),
                name: msg.name().map(|n| n.to_string()),
                tool_call_id: msg.tool_call_id(),
                tool_calls: match msg {
                    ChatCompletionRequestMessage::Assistant { tool_calls, .. } => {
                        match tool_calls {
                            Some(tcs) => {
                                Some(tcs.iter().map(|tc| tc.clone().into()).collect::<Vec<openrouter_api::models::tool::ToolCall>>())
                            },
                            None => None
                        }
                    },
                    _ => None
                },
            }
        }).collect();

        let request = ChatCompletionRequest {
            model: self.props.request.model.clone(),
            messages,
            stream: Some(false),
            response_format: self.props.request.response_format.clone().map(|rf| rf.into()),
            tools: self.props.request.tools.clone().map(|vt| vt.into_iter().map(|t| t.into()).collect::<Vec<Tool>>()),
            provider: None,
            models: None,
            transforms: None,
        };

        let response = self.client.chat_completion(request).await?;
        Ok(response.into())
    }

    pub async fn execute_chat_stream(
        &self,
        tx: Sender<Result<LlmServiceChatCompletionChunk, LlmStreamingError>>,
    ) -> Result<LlmServiceChatCompletionResponse, LlmError> {
        let messages: Vec<openrouter_api::types::chat::Message> = self.props.request.messages.iter().map(|msg| {
            openrouter_api::types::chat::Message {
                role: msg.role().to_string(),
                content: msg.content(),
                name: msg.name().map(|n| n.to_string()),
                tool_call_id: msg.tool_call_id(),
                tool_calls: match msg {
                    ChatCompletionRequestMessage::Assistant { tool_calls, .. } => {
                        match tool_calls {
                            Some(tcs) => {
                                Some(tcs.iter().map(|tc| tc.clone().into()).collect::<Vec<openrouter_api::models::tool::ToolCall>>())
                            },
                            None => None
                        }
                    },
                    _ => None
                },
            }
        }).collect();

        let request = ChatCompletionRequest {
            model: self.props.request.model.clone(),
            messages,
            stream: Some(true),
            response_format: self.props.request.response_format.clone().map(|rf| rf.into()),
            tools: self.props.request.tools.clone().map(|vt| vt.into_iter().map(|t| t.into()).collect::<Vec<Tool>>()),
            provider: None,
            models: None,
            transforms: None,
        };

        let mut stream = self.client.chat()?.chat_completion_stream(request);
        let mut content: Option<String> = None;
        let mut prompt_tokens = 0;
        let mut completion_tokens = 0;
        let mut total_tokens = 0;
        let mut id = String::new();

        while let Some(chunk) = stream.next().await {
            tracing::debug!("chunk: {:?}", chunk);
            match chunk {
                Ok(c) => {
                    id = c.id.clone();

                    if let Some(u) = &c.usage {
                        completion_tokens = u.completion_tokens;         
                        prompt_tokens = u.prompt_tokens;
                        total_tokens = u.total_tokens;
                    }

                    if let Some(c) = &c.choices.first() {
                        if let Some(c) = &c.delta.content {
                            match &mut content {
                                Some(cnt) => cnt.push_str(&c),
                                None => content = Some(c.to_string())
                            }
                        }
                    }

                    // TODO: Capture tool calls

                    if let Err(_) = tx.send(Ok(c.into())).await {
                        break;
                    }
                }
                Err(e) => eprintln!("Error during streaming: {}", e),
            }
        }

        let _ = tx.send(Ok(LlmServiceChatCompletionChunk::done_sentinel(id.clone()))).await;
    
        let created = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs() as i64;

        Ok(LlmServiceChatCompletionResponse::new_streamed(
            id, 
            content, 
            self.props.request.model.clone(),
            created, 
            Some(prompt_tokens), 
            Some(completion_tokens), 
            Some(total_tokens),
            None, // OpenRouter doesn't provide prompt_tokens_details yet
            None, // OpenRouter doesn't provide completion_tokens_details yet
        ))
    }
}
