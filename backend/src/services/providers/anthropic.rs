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

pub struct AnthropicProvider<'a> {
    props: &'a LlmProps,
    streaming: bool,
}

impl<'a> AnthropicProvider<'a> {
    pub fn new(props: &'a LlmProps, streaming: bool) -> Self {
        AnthropicProvider {
            props,
            streaming,
        }
    }
}

// response structs
#[derive(Deserialize, Serialize, Clone)]
struct AnthropicResponse {
    content: Vec<AnthropicMessage>,
    id: String,
    model: String,
    role: String,
    #[serde(rename = "stop_reason")]
    stop_reason: String,
    #[serde(rename = "stop_sequence")]
    stop_sequence: Option<String>,
    #[serde(rename = "type")]
    message_type: String,
    usage: AnthropicUsage,
}

#[derive(serde::Deserialize, Serialize, Clone)]
struct AnthropicMessage {
    text: String,
    #[serde(rename = "type")] 
    content_type: String,
}

#[derive(serde::Deserialize, Serialize, Clone)]
struct AnthropicUsage {
    #[serde(rename = "input_tokens")]
    input_tokens: u32,
    #[serde(rename = "output_tokens")] 
    output_tokens: u32,
}


// STREAMING RESPONE
#[derive(Debug, Deserialize, Serialize)]
struct AnthropicResponseStreamChunk {
    #[serde(rename = "type")]
    event_type: String,
    delta: Option<AnthropicResponseStreamDelta>,
    content_block: Option<AnthropicResponseStreamContentBlock>,
    usage: Option<AnthropicResponseStreamUsage>,
    message: Option<AnthropicResponseStreamMessage>
}

#[derive(Debug, Deserialize, Serialize)]
struct AnthropicResponseStreamMessage {
    id: String,
    usage: AnthropicResponseStreamUsage
}

#[derive(Debug, Deserialize, Serialize)]
struct AnthropicResponseStreamDelta {
    text: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct AnthropicResponseStreamContentBlock {
    text: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct AnthropicResponseStreamUsage {
    input_tokens: Option<i64>,
    output_tokens: Option<i64>,
}

impl From<AnthropicResponse> for LlmApiResponseProps {
    fn from(response: AnthropicResponse) -> Self {
        let response_text = response.content
            .first()
            .map(|msg| msg.text.clone())
            .unwrap_or_default();

        LlmApiResponseProps {
            response_content: response_text,
            raw_response: serde_json::to_string(&response).unwrap_or_default(),
            input_tokens: Some(response.usage.input_tokens as i64),
            output_tokens: Some(response.usage.output_tokens as i64),
            reasoning_tokens: None
        }
    }
}

impl<'a> LlmProvider for AnthropicProvider<'a> {
    fn build_request(&self) -> Result<(RequestBuilder, String), Error> {
        let client = reqwest::Client::new();
        let api_key = std::env::var("ANTHROPIC_API_KEY").map_err(|_| Error::Auth)?;
        let body = self.create_body();
        let body_string = body.to_string();

        let request = client
            .post(format!("{}/messages", &self.props.base_url))
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body);

        Ok((request, body_string))
    }

    fn parse_response(json_text: &str) -> Result<LlmApiResponseProps, Error> {
        let response: AnthropicResponse = serde_json::from_str(json_text)?;
        Ok(response.into())
    }

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
                            match serde_json::from_str::<AnthropicResponseStreamChunk>(&message.data) {
                                Ok(chunk) => {
                                    match chunk.event_type.as_str() {
                                        "content_block_start" => {
                                            if let Some(content_block) = &chunk.content_block {
                                                if let Some(text) = &content_block.text {
                                                    stream_content += &text;
                                                    if let Err(_) = tx.send(Ok(text.clone())).await {
                                                        break; // Receiver dropped
                                                    }
                                                }
                                            }
                                        }
                                        "content_block_delta" => {
                                            if let Some(delta) = &chunk.delta {
                                                if let Some(text) = &delta.text {
                                                    stream_content += &text;
                                                    if let Err(_) = tx.send(Ok(text.clone())).await {
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                        "message_start" => {
                                            if let Some(message) = &chunk.message {
                                                if let Some(input) = message.usage.input_tokens {
                                                    input_tokens += input;
                                                }
                                            }
                                            chunks.push(chunk); // captures start chunk
                                        }
                                        "message_delta" => {
                                            if let Some(usage) = &chunk.usage {
                                                if let Some(output) = usage.output_tokens {
                                                    output_tokens += output
                                                }
                                            }
                                            chunks.push(chunk); // captures end chunk
                                        }
                                        "message_stop" => {
                                            let _ = tx.send(Ok("[DONE]".to_string())).await;
                                            break;
                                        }
                                        _ => {} // Ignore other event types
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

            return LlmApiResponseProps {
                response_content: stream_content,
                raw_response,
                input_tokens: Some(input_tokens),
                output_tokens: Some(output_tokens),
                reasoning_tokens: None
            };
        }).await?;


        Ok(result)
    }

    fn create_body(&self) -> serde_json::Value {
        let system_content = self.props.messages.iter()
            .find_map(|msg| match msg {
                Message::System { content } => Some(content.as_str()),
                _ => None
            });

        // Convert conversation history to Anthropic's format
        let filtered_messages: Vec<serde_json::Value> = self.props.messages.iter()
            .filter_map(|msg| match msg {
                Message::System { .. } => None,
                Message::User { content } => Some(json!({
                    "role": "user",
                    "content": content
                })),
                Message::Assistant { content } => Some(json!({
                    "role": "assistant",
                    "content": content
                })),
            })
            .collect();

        let mut body = json!({
            "messages": filtered_messages
        });

        if let Some(content) = system_content {
            body["system"] = json!(content);
        }

        body["model"] = json!(self.props.model_name);
        body["stream"] = json!(self.streaming);
        body["temperature"] = json!(self.props.temperature);
        body["max_tokens"] = json!(self.props.max_tokens);

        body
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_anthropic_response_parsing() {
        let response = json!({
            "content": [{
                "text": "test response",
                "type": "text"
            }],
            "id": "msg_some_id",
            "model": "claude-v1",
            "role": "assistant",
            "stop_reason": "end_turn",
            "stop_sequence": null,
            "type": "message",
            "usage": {
                "input_tokens": 100,
                "output_tokens": 50
            }
        })
        .to_string();

        let result = AnthropicProvider::parse_response(&response).unwrap();
        let result: LlmApiResponseProps = result.into();
        assert_eq!(result.response_content, "test response");
        assert_eq!(result.input_tokens, Some(100));
        assert_eq!(result.output_tokens, Some(50));
    }
}

