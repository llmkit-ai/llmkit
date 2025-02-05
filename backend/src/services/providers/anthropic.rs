use crate::services::{
    llm::{Error, LlmProvider}, 
    types::{llm_props::LlmProps, message::Message, stream::LlmStreamingError}
};

use anyhow::Result;
use reqwest::RequestBuilder;
use reqwest_eventsource::Event;
use serde_json::json;
use tokio::sync::mpsc::Sender;
use futures_util::StreamExt;

pub struct AnthropicProvider<'a> {
    props: &'a LlmProps,
    streaming: bool
}

impl<'a> AnthropicProvider<'a> {
    pub fn new(props: &'a LlmProps, streaming: bool) -> Self {
        AnthropicProvider {
            props,
            streaming
        }
    }
}


impl<'a> LlmProvider for AnthropicProvider<'a> {
    fn build_request(&self) -> Result<RequestBuilder, Error> {
        let client = reqwest::Client::new();
        let api_key = std::env::var("ANTHROPIC_API_KEY").map_err(|_| Error::Auth)?;
        let body = self.create_body();

        Ok(client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body))
    }

    fn parse_response(json_text: &str) -> Result<String, Error> {
        println!("{:?}", json_text);

        #[derive(serde::Deserialize)]
        struct ResponseJson {
            content: Vec<MessageContent>,
        }
        #[derive(serde::Deserialize)]
        struct MessageContent {
            text: String,
        }

        let response: ResponseJson = serde_json::from_str(json_text)?;
        response
            .content
            .first()
            .and_then(|c| Some(c.text.clone()))
            .ok_or(Error::Provider("Empty Anthropic response".into()))
    }

    fn log_response(&self, request_text: &str, response_text: &str) -> Result<(), Error> {
        todo!()         
    }

    fn stream_eventsource(
        mut event_source: reqwest_eventsource::EventSource, 
        tx: Sender<Result<String, LlmStreamingError>>
    ) {
        #[derive(Debug, serde::Deserialize)]
        struct AnthropicResponseChunk {
            #[serde(rename = "type")]
            event_type: String,
            delta: Option<Delta>,
            content_block: Option<ContentBlock>,
        }

        #[derive(Debug, serde::Deserialize)]
        struct Delta {
            text: Option<String>,
        }

        #[derive(Debug, serde::Deserialize)]
        struct ContentBlock {
            text: Option<String>,
        }

        tokio::spawn(async move {
            while let Some(event_result) = event_source.next().await {
                match event_result {
                    Ok(event) => {
                        if let Event::Message(message) = event {
                            match serde_json::from_str::<AnthropicResponseChunk>(&message.data) {
                                Ok(chunk) => {
                                    match chunk.event_type.as_str() {
                                        "content_block_start" => {
                                            if let Some(content_block) = chunk.content_block {
                                                if let Some(text) = content_block.text {
                                                    if let Err(_) = tx.send(Ok(text)).await {
                                                        break; // Receiver dropped
                                                    }
                                                }
                                            }
                                        }
                                        "content_block_delta" => {
                                            if let Some(delta) = chunk.delta {
                                                if let Some(text) = delta.text {
                                                    if let Err(_) = tx.send(Ok(text)).await {
                                                        break;
                                                    }
                                                }
                                            }
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
        });
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

        let model: String = self.props.model.clone().into();

        body["model"] = json!(model);
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
                "text": "test response"
            }]
        })
        .to_string();

        let result = AnthropicProvider::parse_response(&response);
        assert_eq!(result.unwrap(), "test response");
    }
}
