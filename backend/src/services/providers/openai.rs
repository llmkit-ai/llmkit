use crate::common::types::chat_request::ChatCompletionRequestMessage;
use crate::common::types::chat_response::{
    CompletionTokensDetails, LlmServiceChatCompletionChunk, LlmServiceChatCompletionResponse,
    PromptTokensDetails,
};

use crate::services::types::{
    llm_error::LlmError, llm_error::LlmStreamingError, llm_service::LlmServiceRequest,
};

use async_openai::config::AzureConfig;
use async_openai::types::{
    ChatCompletionMessageToolCall, ChatCompletionRequestToolMessageArgs, ChatCompletionRequestToolMessageContent,
    ChatCompletionStreamOptions, ChatCompletionTool, ReasoningEffort, ResponseFormat
};

use async_openai::{
    config,
    types::{
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
    },
    Client,
};

use anyhow::Result;
use futures_util::StreamExt;
use tokio::sync::mpsc::Sender;

use std::time::{SystemTime, UNIX_EPOCH};

enum ClientConfigs { 
    OpenAi(async_openai::Client<config::OpenAIConfig>),
    Azure(async_openai::Client<config::AzureConfig>),
}

pub struct OpenAiProvider<'a> {
    props: &'a LlmServiceRequest,
    client: ClientConfigs,
}

impl<'a> OpenAiProvider<'a> {
    pub fn new(props: &'a LlmServiceRequest) -> Result<Self, LlmError> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| LlmError::InvalidConfig("Missing OPENAI_API_KEY".to_string()))?;

        let config = config::OpenAIConfig::new().with_api_key(api_key);

        let client = Client::with_config(config);

        Ok(OpenAiProvider {
            props,
            client: ClientConfigs::OpenAi(client),
        })
    }

    pub fn new_azure(props: &'a LlmServiceRequest) -> Result<Self, LlmError> {
        let api_key = std::env::var("AZURE_API_KEY")
            .map_err(|_| LlmError::InvalidConfig("Missing AZURE_API_KEY".to_string()))?;

        let base_url = props.base_url.clone()
            .ok_or_else(|| LlmError::InvalidConfig("Missing AZURE_BASE_URL".to_string()))?;

        let mut model_parts = props.request.model.split("|");
        let model_name = model_parts.next()
            .ok_or_else(|| LlmError::InvalidConfig("Invalid model name config missing name".to_string()))?;

        let model_api_version = model_parts.next()
            .ok_or_else(|| LlmError::InvalidConfig("Invalid model name config missing version".to_string()))?;

        let config = AzureConfig::new()
            .with_api_base(base_url)
            .with_api_key(api_key)
            .with_deployment_id(model_name)
            .with_api_version(model_api_version);

        let client = Client::with_config(config);

        Ok(OpenAiProvider {
            props,
            client: ClientConfigs::Azure(client),
        })
    }

    pub async fn execute_chat(&self) -> Result<LlmServiceChatCompletionResponse, LlmError> {
        let mut messages: Vec<async_openai::types::ChatCompletionRequestMessage> = vec![];

        for msg in self.props.request.messages.iter() {
            match msg {
                ChatCompletionRequestMessage::System { content, name: _ } => messages.push(
                    ChatCompletionRequestSystemMessageArgs::default()
                        .content(content.clone())
                        .build()?
                        .into(),
                ),
                ChatCompletionRequestMessage::User { content, name: _ } => messages.push(
                    ChatCompletionRequestUserMessageArgs::default()
                        .content(content.clone())
                        .build()?
                        .into(),
                ),
                ChatCompletionRequestMessage::Assistant {
                    content,
                    tool_calls,
                    name: _,
                } => {
                    let mut assistant_message =
                        ChatCompletionRequestAssistantMessageArgs::default();
                    assistant_message.content(content.clone().unwrap_or("".to_string()));

                    if let Some(tcs) = tool_calls {
                        let oai_tool_calls = tcs
                            .clone()
                            .into_iter()
                            .map(|tc| tc.into())
                            .collect::<Vec<ChatCompletionMessageToolCall>>();
                        assistant_message.tool_calls(oai_tool_calls);
                    }

                    messages.push(assistant_message.build()?.into());
                }
                ChatCompletionRequestMessage::Tool {
                    content,
                    tool_call_id,
                } => {
                    let tool_message: ChatCompletionRequestToolMessageContent =
                        ChatCompletionRequestToolMessageContent::Text(content.clone());

                    messages.push(
                        ChatCompletionRequestToolMessageArgs::default()
                            .content(tool_message)
                            .tool_call_id(tool_call_id.clone())
                            .build()?
                            .into(),
                    );
                }
            }
        }

        let oai_tools = self.props.request.tools.clone().map(|vt| {
            vt.into_iter()
                .map(|t| t.into())
                .collect::<Vec<ChatCompletionTool>>()
        });
        let repsonse_format: Option<ResponseFormat> = self
            .props
            .request
            .response_format
            .clone()
            .map(|rf| rf.into());

        let mut request = CreateChatCompletionRequestArgs::default();

        if self.props.is_reasoning {
            if let Some(max_tokens) = self.props.request.max_tokens {
                request.max_completion_tokens(max_tokens * 2);
            }
            
            // Set reasoning effort based on prompt configuration
            let reasoning_effort = match self.props.reasoning_effort.as_deref() {
                Some("low") => ReasoningEffort::Low,
                Some("medium") => ReasoningEffort::Medium,
                Some("high") => ReasoningEffort::High,
                _ => ReasoningEffort::Low, // Default to low if not specified
            };
            request.reasoning_effort(reasoning_effort);
        } else {
            if let Some(max_tokens) = self.props.request.max_tokens {
                request.max_tokens(max_tokens);
            }
            if let Some(temperature) = self.props.request.temperature {
                request.temperature(temperature);
            }
        }

        request.model(self.props.request.model.clone());
        request.messages(messages);
        request.stream(false);

        if let Some(tools) = oai_tools {
            request.tools(tools);
        }

        if let Some(rf) = repsonse_format {
            request.response_format(rf);
        }

        let request = request.build()?;

        println!("request: {}", serde_json::to_string(&request).unwrap());

        match &self.client {
            ClientConfigs::OpenAi(c) => {
                let response = c.chat().create(request).await?;
                Ok(response.into())
            },
            ClientConfigs::Azure(c) => {
                let response = c.chat().create(request).await?;
                Ok(response.into())
            },
        }
    }

    pub async fn execute_chat_stream(
        &self,
        tx: Sender<Result<LlmServiceChatCompletionChunk, LlmStreamingError>>,
    ) -> Result<LlmServiceChatCompletionResponse, LlmError> {
        let mut messages: Vec<async_openai::types::ChatCompletionRequestMessage> = vec![];

        for msg in self.props.request.messages.iter() {
            match msg {
                ChatCompletionRequestMessage::System { content, name: _ } => messages.push(
                    ChatCompletionRequestSystemMessageArgs::default()
                        .content(content.clone())
                        .build()?
                        .into(),
                ),
                ChatCompletionRequestMessage::User { content, name: _ } => messages.push(
                    ChatCompletionRequestUserMessageArgs::default()
                        .content(content.clone())
                        .build()?
                        .into(),
                ),
                ChatCompletionRequestMessage::Assistant {
                    content,
                    tool_calls,
                    name: _,
                } => {
                    let mut assistant_message =
                        ChatCompletionRequestAssistantMessageArgs::default();
                    assistant_message.content(content.clone().unwrap_or("".to_string()));

                    if let Some(tcs) = tool_calls {
                        let oai_tool_calls = tcs
                            .clone()
                            .into_iter()
                            .map(|tc| tc.into())
                            .collect::<Vec<ChatCompletionMessageToolCall>>();
                        assistant_message.tool_calls(oai_tool_calls);
                    }

                    messages.push(assistant_message.build()?.into());
                }
                ChatCompletionRequestMessage::Tool {
                    content,
                    tool_call_id,
                } => {
                    let tool_message: ChatCompletionRequestToolMessageContent =
                        ChatCompletionRequestToolMessageContent::Text(content.clone());

                    messages.push(
                        ChatCompletionRequestToolMessageArgs::default()
                            .content(tool_message)
                            .tool_call_id(tool_call_id.clone())
                            .build()?
                            .into(),
                    );
                }
            }
        }

        let oai_tools = self.props.request.tools.clone().map(|vt| {
            vt.into_iter()
                .map(|t| t.into())
                .collect::<Vec<ChatCompletionTool>>()
        });

        let repsonse_format: Option<ResponseFormat> = self
            .props
            .request
            .response_format
            .clone()
            .map(|rf| rf.into());

        let mut request = CreateChatCompletionRequestArgs::default();

        if self.props.is_reasoning {
            if let Some(max_tokens) = self.props.request.max_tokens {
                request.max_completion_tokens(max_tokens * 2);
            }
            
            // Set reasoning effort based on prompt configuration
            let reasoning_effort = match self.props.reasoning_effort.as_deref() {
                Some("low") => ReasoningEffort::Low,
                Some("medium") => ReasoningEffort::Medium,
                Some("high") => ReasoningEffort::High,
                _ => ReasoningEffort::Low, // Default to low if not specified
            };
            request.reasoning_effort(reasoning_effort);
        } else {
            if let Some(max_tokens) = self.props.request.max_tokens {
                request.max_tokens(max_tokens);
            }
            if let Some(temperature) = self.props.request.temperature {
                request.temperature(temperature);
            }
        }

        request.model(self.props.request.model.clone());
        request.messages(messages);
        request.stream(true);
        request.stream_options(ChatCompletionStreamOptions {
            include_usage: true,
        });

        if let Some(tools) = oai_tools {
            request.tools(tools);
        }

        if let Some(rf) = repsonse_format {
            request.response_format(rf);
        }

        let request = request.build()?;

        let mut stream = match &self.client {
            ClientConfigs::OpenAi(c) => {
                c.chat().create_stream(request).await?
            },
            ClientConfigs::Azure(c) => {
                c.chat().create_stream(request).await?
            },
        };

        let mut content: Option<String> = None;
        let mut prompt_tokens = 0;
        let mut completion_tokens = 0;
        let mut total_tokens = 0;
        let mut prompt_tokens_details: Option<PromptTokensDetails> = None;
        let mut completion_tokens_details: Option<CompletionTokensDetails> = None;
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
                        
                        // Capture token details if available
                        prompt_tokens_details = u.prompt_tokens_details.as_ref().map(|details| PromptTokensDetails {
                            audio_tokens: details.audio_tokens,
                            cached_tokens: details.cached_tokens,
                        });
                        
                        completion_tokens_details = u.completion_tokens_details.as_ref().map(|details| CompletionTokensDetails {
                            accepted_prediction_tokens: details.accepted_prediction_tokens,
                            audio_tokens: details.audio_tokens,
                            reasoning_tokens: details.reasoning_tokens,
                            rejected_prediction_tokens: details.rejected_prediction_tokens,
                        });
                    }

                    if let Some(c) = &c.choices.first() {
                        if let Some(c) = &c.delta.content {
                            match &mut content {
                                Some(cnt) => cnt.push_str(&c),
                                None => content = Some(c.to_string()),
                            }
                        }
                    }

                    // TODO: Capture tool calls

                    if let Err(_) = tx.send(Ok(c.into())).await {
                        break;
                    }
                }
                Err(e) => {
                    tracing::error!("OpenAI Error during streaming: {}", e);
                    return Err(e.into());
                },
            }
        }

        let _ = tx
            .send(Ok(LlmServiceChatCompletionChunk::done_sentinel(id.clone())))
            .await;

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
            prompt_tokens_details,
            completion_tokens_details,
        ))
    }
}
