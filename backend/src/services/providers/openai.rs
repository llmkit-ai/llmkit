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

pub struct OpenaiProvider;

impl LlmProvider for OpenaiProvider {
    fn build_request(props: &LlmProps, streaming: bool) -> Result<RequestBuilder, Error> {
        let client = reqwest::Client::new();
        let api_key = std::env::var("OPENAI_API_KEY").map_err(|_| Error::Auth)?;

        // Convert messages to OpenAI format
        let messages = props.messages.iter()
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

        let model: String = props.model.clone().into();
        let mut body = json!({
            "model": model,
            "messages": messages,
            "stream": streaming,
            "temperature": props.temperature,
            "max_completion_tokens": props.max_tokens
        });

        if props.json_mode {
            body["response_format"] = json!({ "type": "json_object" });
        }

        Ok(client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&body))
    
    }

    fn parse_response(json_text: &str) -> Result<String, Error> {
        #[derive(serde::Deserialize)]
        struct ResponseJson {
            choices: Vec<ResponseJsonChoice>,
        }
        #[derive(serde::Deserialize)]
        struct ResponseJsonChoice {
            message: MessageContent,
        }
        #[derive(serde::Deserialize)]
        struct MessageContent {
            content: String,
        }

        let response: ResponseJson = serde_json::from_str(json_text)?;
        response
            .choices
            .first()
            .and_then(|c| Some(c.message.content.clone()))
            .ok_or(Error::Provider("Empty OpenAI response".into()))
    
    }

    fn stream_eventsource(
        mut event_source: reqwest_eventsource::EventSource, 
        tx: Sender<Result<String, LlmStreamingError>>
    ) {
        #[derive(Debug, serde::Deserialize)]
        struct OpenAiResponseChunk {
            choices: Vec<Choice>,
        }
        #[derive(Debug, serde::Deserialize)]
        struct Choice {
            delta: Delta,
        }
        #[derive(Debug, serde::Deserialize)]
        struct Delta {
            content: Option<String>,
        }

        tokio::spawn(async move {
            while let Some(event_result) = event_source.next().await {
                match event_result {
                    Ok(event) => {
                        if let Event::Message(message) = event {
                            if message.data == "[DONE]" {
                                let _ = tx.send(Ok("[DONE]".to_string())).await;
                                break;
                            }

                            match serde_json::from_str::<OpenAiResponseChunk>(&message.data) {
                                Ok(chunk) => {
                                    if let Some(content) = chunk.choices
                                        .first()
                                        .and_then(|c| c.delta.content.as_ref())
                                    {
                                        if let Err(_) = tx.send(Ok(content.clone())).await {
                                            break; // Receiver dropped
                                        }
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
}


#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Unit tests for response parsing
    #[test]
    fn test_openai_response_parsing() {
        let response = json!({
            "choices": [{
                "message": {
                    "content": "test response"
                }
            }]
        })
        .to_string();

        let result = OpenaiProvider::parse_response(&response);
        assert_eq!(result.unwrap(), "test response");
    }
}
