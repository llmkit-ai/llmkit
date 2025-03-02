use crate::services::types::chat_response::{LlmServiceChatCompletionChunk, LlmServiceChatCompletionResponse};
use crate::services::types::chat_request::Message;
use crate::services::types::{
    llm_error::LlmError, llm_error::LlmStreamingError, chat_request::LlmServiceRequest
};
use anyhow::Result;
use futures_util::StreamExt;
use openrouter_api::models::tool::ToolCall;
use openrouter_api::{OpenRouterClient, Ready};
use openrouter_api::types::chat::ChatCompletionRequest;
use tokio::sync::mpsc::Sender;

use std::time::{SystemTime, UNIX_EPOCH};


pub struct OpenrouterProvider<'a> {
    props: &'a LlmServiceRequest,
    streaming: bool,
    client: OpenRouterClient<Ready>,
}

impl<'a> OpenrouterProvider<'a> {
    /// Creates a new instance of `OpenrouterProvider` with the given properties and streaming flag.
    pub fn new(props: &'a LlmServiceRequest, streaming: bool) -> Result<Self, LlmError> {
        let api_key = std::env::var("OPENROUTER_API_KEY").expect("Missing OPENROUTER_API_KEY");
        let client = OpenRouterClient::new()
            .with_base_url("https://openrouter.ai/api/v1/")?
            .with_api_key(api_key)?;

        Ok(OpenrouterProvider {
            props,
            streaming,
            client,
        })
    }

    /// Builds an HTTP request using the OpenRouter API library's client configuration.
    pub async fn execute_chat(&self) -> Result<LlmServiceChatCompletionResponse, LlmError> {
        let messages = self.props.messages.iter().map(|msg| {
            openrouter_api::types::chat::Message {
                role: match msg {
                    Message::System { .. } => "system".to_string(),
                    Message::User { .. } => "user".to_string(),
                    Message::Assistant { .. } => "assistant".to_string(),
                },
                content: match msg {
                    Message::System { content } | Message::User { content } | Message::Assistant { content, .. } => content.clone(),
                },
                name: None,
                tool_calls: match msg {
                    Message::Assistant { tool_calls, .. } => {
                        match tool_calls {
                            Some(tcs) => {
                                Some(tcs.iter().map(|tc| tc.to_owned().into()).collect::<Vec<ToolCall>>())
                            },
                            None => None
                        }
                    },
                    _ => None
                },
            }
        }).collect();

        let request = ChatCompletionRequest {
            model: self.props.model_name.clone(),
            messages,
            stream: if self.streaming { Some(true) } else { None },
            response_format: if self.props.json_mode { Some("{\"type\": \"json_object\"}".to_string()) } else { None },
            tools: None,
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
        let messages: Vec<openrouter_api::types::chat::Message> = self.props.messages.iter().map(|msg| {
            openrouter_api::types::chat::Message {
                role: match msg {
                    Message::System { .. } => "system".to_string(),
                    Message::User { .. } => "user".to_string(),
                    Message::Assistant { .. } => "assistant".to_string(),
                },
                content: match msg {
                    Message::System { content } | Message::User { content } | Message::Assistant { content, .. } => content.clone(),
                },
                name: None,
                tool_calls: match msg {
                    Message::Assistant { tool_calls, .. } => {
                        match tool_calls {
                            Some(tcs) => {
                                Some(tcs.iter().map(|tc| tc.to_owned().into()).collect::<Vec<ToolCall>>())
                            },
                            None => None
                        }
                    },
                    _ => None
                },
            }
        }).collect();

        let request = ChatCompletionRequest {
            model: self.props.model_name.clone(),
            messages,
            stream: Some(true),
            response_format: None,
            tools: None,
            provider: None,
            models: None,
            transforms: None,
        };

        let mut stream = self.client.chat()?.chat_completion_stream(request);
        let mut content = String::new();
        let mut prompt_tokens = 0;
        let mut completion_tokens = 0;
        let mut total_tokens = 0;
        let mut id = String::new();

        while let Some(chunk) = stream.next().await {
            tracing::info!("chunk: {:?}", chunk);
            match chunk {
                Ok(c) => {
                    id = c.id.clone();

                    if let Some(u) = &c.usage {
                        completion_tokens = u.completion_tokens;         
                        prompt_tokens = u.prompt_tokens;
                        total_tokens = u.total_tokens;
                    }

                    if let Some(c) = &c.choices.first() {
                        content += &c.delta.content;
                    }

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
            self.props.model_name.clone(),
            created, 
            Some(prompt_tokens), 
            Some(completion_tokens), 
            Some(total_tokens)
        ))
    }
}
