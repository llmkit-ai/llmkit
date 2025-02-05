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

pub struct DeepseekProvider<'a> {
    props: &'a LlmProps,
    streaming: bool
}

#[derive(serde::Deserialize)]
struct ResponseJson {
    choices: Vec<ResponseChoice>,
}
#[derive(serde::Deserialize)]
struct ResponseChoice {
    message: MessageContent,
}
#[derive(serde::Deserialize)]
struct MessageContent {
    content: String,
}

impl<'a> DeepseekProvider<'a> {
    pub fn new(props: &'a LlmProps, streaming: bool) -> Self {
        DeepseekProvider {
            props,
            streaming
        }
    }
}

impl<'a> LlmProvider for DeepseekProvider<'a> {
    fn build_request(&self) -> Result<RequestBuilder, Error> {
        let client = reqwest::Client::new();
        let api_key = std::env::var("DEEPSEEK_API_KEY").map_err(|_| Error::Auth)?;

        let body = self.create_body();

        Ok(client
            .post("https://api.deepseek.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&body))
    }

    fn parse_response(json_text: &str) -> Result<String, Error> {
        let response: ResponseJson = serde_json::from_str(json_text)?;
        response
            .choices
            .first()
            .and_then(|c| Some(c.message.content.clone()))
            .ok_or(Error::Provider("Empty Deepseek response".into()))

    }

    fn log_response(&self, request_text: &str, response_text: &str) -> Result<(), Error> {
        todo!()
    }

    fn stream_eventsource(
        mut event_source: reqwest_eventsource::EventSource, 
        tx: Sender<Result<String, LlmStreamingError>>
    ) {
        #[derive(Debug, serde::Deserialize)]
        struct DeepseekResponseChunk {
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

                            match serde_json::from_str::<DeepseekResponseChunk>(&message.data) {
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

        let mut body = json!({
            "model": model,
            "messages": messages,
            "stream": self.streaming,
            "temperature": self.props.temperature,
            "max_completion_tokens": self.props.max_tokens
        });

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
    fn test_deepseek_response_parsing() {
        let response = json!({
            "choices": [{
                "message": {
                    "content": "test response"
                }
            }]
        })
        .to_string();

        let result = DeepseekProvider::parse_response(&response);
        assert_eq!(result.unwrap(), "test response");
    }
}
