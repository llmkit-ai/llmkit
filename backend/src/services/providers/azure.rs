use crate::services::{
    llm::{Error, LlmProvider}, 
    types::{llm_props::LlmProps, message::Message, parse_response::LlmApiResponseProps, stream::LlmStreamingError}
};

use anyhow::Result;
use reqwest::RequestBuilder;
use reqwest_eventsource::Event;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::mpsc::Sender;
use futures_util::StreamExt;

pub struct AzureProvider<'a> {
    props: &'a LlmProps,
    streaming: bool,
}

impl<'a> AzureProvider<'a> {
    pub fn new(props: &'a LlmProps, streaming: bool) -> Self {
        AzureProvider { props, streaming }
    }
}

/// Updated structs matching the Azure API response.
///
/// The new response JSON looks roughly like:
///
/// ```json
/// {
///   "id": "chatcmpl-Azx9z8MeysrOJq4siPYc7nSe0FLuw",
///   "object": "chat.completion",
///   "created": 1739328783,
///   "model": "gpt-4mini-2024-07-18",
///   "system_fingerprint": "fp_f3927aa00d",
///   "choices": [
///     {
///       "index": 0,
///       "finish_reason": "stop",
///       "logprobs": null,
///       "message": {
///         "content": "Hello! I'm just a computer program, so I don't have feelings, but I'm here and ready to help you. How can I assist you today?",
///         "refusal": null,
///         "role": "assistant"
///       }
///     }
///   ],
///   "usage": {
///     "prompt_tokens": 23,
///     "completion_tokens": 30,
///     "total_tokens": 53
///   }
/// }
/// ```
///
#[derive(Deserialize, Serialize, Clone)]
pub struct AzureResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub system_fingerprint: String,
    pub choices: Vec<AzureResponseChoice>,
    pub usage: AzureUsage,
    // Note: The JSON may also include a "prompt_filter_results" field, which we ignore here.
}

#[derive(Deserialize, Serialize, Clone)]
pub struct AzureResponseChoice {
    pub index: i32,
    pub finish_reason: Option<String>,
    pub logprobs: Option<serde_json::Value>,
    pub message: AzureMessage,
    // Note: A "content_filter_results" field may be present but is not used.
}

#[derive(Deserialize, Serialize, Clone)]
pub struct AzureMessage {
    pub content: String,
    pub refusal: Option<String>,
    pub role: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct AzureUsage {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
    // Extra usage details (like *_details) will be ignored.
}


// Structures for streaming responses.
#[derive(Deserialize, Serialize, Clone)]
struct AzureResponseStreamChunk {
    choices: Vec<AzureResponseStreamChoice>,
    usage: Option<AzureResponseStreamUsage>,
}

#[derive(Deserialize, Serialize, Clone)]
struct AzureResponseStreamChoice {
    delta: AzureResponseStreamDelta,
}

#[derive(Deserialize, Serialize, Clone)]
struct AzureResponseStreamDelta {
    content: Option<String>,
}

#[derive(Deserialize, Serialize, Clone)]
struct AzureResponseStreamUsage {
    completion_tokens: i64,
    prompt_tokens: i64,
    total_tokens: i64,
}

/// Convert an `AzureResponse` into the generic `LlmApiResponseProps`.
impl From<AzureResponse> for LlmApiResponseProps {
    fn from(response: AzureResponse) -> Self {
        // Use the content from the first choice, if available.
        let response_content = response
            .choices
            .first()
            .map(|choice| choice.message.content.clone())
            .unwrap_or_default();

        // Serialize the full response into a string for debugging/logging.
        let raw_response = serde_json::to_string(&response).unwrap_or_default();

        let input_tokens = Some(response.usage.prompt_tokens as i64);
        let output_tokens = Some(response.usage.completion_tokens as i64);

        LlmApiResponseProps {
            response_content,
            raw_response,
            input_tokens,
            output_tokens,
            reasoning_tokens: None,
        }
    }
}

impl<'a> LlmProvider for AzureProvider<'a> {
    /// Build a request using Azure’s endpoint and authentication.
    fn build_request(&self) -> Result<(RequestBuilder, String), Error> {
        let client = reqwest::Client::new();
        let url = std::env::var("AZURE_ENDPOINT").map_err(|_| Error::Auth)?;
        let api_key = std::env::var("AZURE_API_KEY").map_err(|_| Error::Auth)?;
        let body = self.create_body();
        let body_string = body.to_string();

        // Note the use of the "api-key" header instead of "Authorization".
        let request = client
            .post(&url)
            .header("api-key", api_key)
            .json(&body);

        Ok((request, body_string))
    }

    /// Parse a complete (non-streaming) response from Azure.
    fn parse_response(json_text: &str) -> Result<LlmApiResponseProps, Error> {
        let response: AzureResponse = serde_json::from_str(json_text)?;
        Ok(response.into())
    }

    /// Process a streaming response via EventSource.
    async fn stream_eventsource(
        mut event_source: reqwest_eventsource::EventSource, 
        tx: Sender<Result<String, LlmStreamingError>>
    ) -> Result<LlmApiResponseProps, Error> {
        let result = tokio::spawn(async move {
            let mut stream_content = String::new();
            let mut output_tokens = 0;
            let mut input_tokens = 0;
            let mut chunks = vec![];

            while let Some(event_result) = event_source.next().await {
                match event_result {
                    Ok(event) => {
                        if let Event::Message(message) = event {
                            if message.data == "[DONE]" {
                                let _ = tx.send(Ok("[DONE]".to_string())).await;
                                break;
                            }

                            match serde_json::from_str::<AzureResponseStreamChunk>(&message.data) {
                                Ok(chunk) => {
                                    if let Some(content) = chunk.choices
                                        .first()
                                        .and_then(|c| c.delta.content.as_ref())
                                    {
                                        stream_content += content;
                                        if let Err(_) = tx.send(Ok(content.to_string())).await {
                                            break;
                                        }
                                    }

                                    if let Some(usage) = &chunk.usage {
                                        output_tokens = usage.completion_tokens;
                                        input_tokens = usage.prompt_tokens;
                                        chunks.push(chunk);
                                    }
                                }
                                Err(e) => {
                                    let _ = tx.send(Err(LlmStreamingError::ParseError(e.to_string()))).await;
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(LlmStreamingError::StreamError(e.to_string()))).await;
                        break;
                    }
                }
            }
            
            event_source.close();

            let raw_response = serde_json::to_string(&chunks)
                .expect("Failed to serialize chunk to string");

            LlmApiResponseProps {
                response_content: stream_content,
                raw_response,
                input_tokens: Some(input_tokens),
                output_tokens: Some(output_tokens),
                reasoning_tokens: None,
            }
        }).await?;

        Ok(result)
    }

    /// Create the request body for Azure.
    /// Note: For Azure, the deployment (and therefore the model) is specified in the URL,
    /// so the "model" parameter is omitted here. Adjust this as needed.
    fn create_body(&self) -> serde_json::Value {
        let model: String = self.props.model.clone().into();
        let messages = self.props.messages.iter()
            .map(|msg| match msg {
                Message::System { content } => json!({
                    "role": "system",
                    "content": content
                }),
                Message::User { content } => json!({
                    "role": "user",
                    "content": content
                }),
                Message::Assistant { content } => json!({
                    "role": "assistant",
                    "content": content
                }),
            })
            .collect::<Vec<_>>();

        // Use "max_tokens" instead of "max_completion_tokens"
        let mut body = json!({
            "messages": messages,
            "temperature": self.props.temperature,
            "max_tokens": self.props.max_tokens,
            "model": model,
        });

        if self.streaming {
            body["stream"] = serde_json::Value::Bool(true);
            body["stream_options"] = json!({
                "include_usage": true
            });
        }

        if self.props.json_mode {
            body["response_format"] = json!({ "type": "json_object" });
        }

        body
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_azure_response_parsing() {
        let response = r#"
        {
            "id": "chatcmpl-Azx9z8MeysrOJq4siPYc7nSe0FLuw",
            "object": "chat.completion",
            "created": 1739328783,
            "model": "gpt-4mini-2024-07-18",
            "system_fingerprint": "fp_f3927aa00d",
            "choices": [
                {
                    "index": 0,
                    "finish_reason": "stop",
                    "logprobs": null,
                    "message": {
                        "content": "Hello! I'm just a computer program, so I don't have feelings, but I'm here and ready to help you. How can I assist you today?",
                        "refusal": null,
                        "role": "assistant"
                    }
                }
            ],
            "usage": {
                "prompt_tokens": 23,
                "completion_tokens": 30,
                "total_tokens": 53
            }
        }
        "#;
        let result = AzureProvider::parse_response(response).unwrap();
        assert_eq!(
            result.response_content,
            "Hello! I'm just a computer program, so I don't have feelings, but I'm here and ready to help you. How can I assist you today?"
        );
        assert_eq!(result.input_tokens, Some(23));
        assert_eq!(result.output_tokens, Some(30));
    }
}
