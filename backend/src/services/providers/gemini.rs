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

pub struct GeminiProvider;

#[derive(serde::Deserialize)]
struct ResponseJson {
    candidates: Vec<ResponseCandidate>,
}

#[derive(serde::Deserialize)]
struct ResponseCandidate {
    content: MessageContent,
}
#[derive(serde::Deserialize)]
struct MessageContent {
    parts: Vec<ContentPart>,
}
#[derive(serde::Deserialize)]
struct ContentPart {
    text: String,
}


impl LlmProvider for GeminiProvider {
    fn build_request(props: &LlmProps, streaming: bool) -> Result<RequestBuilder, Error> {
        let client = reqwest::Client::new();
        let api_key = std::env::var("GOOGLE_API_KEY").map_err(|_| Error::Auth)?;

        // Extract and combine system messages
        let system_instruction = props.messages.iter()
            .filter_map(|msg| {
                if let Message::System { content } = msg {
                    Some(content.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<String>>()
            .join("\n\n");

        // Process conversation history into Gemini's format
        let contents = props.messages.iter()
            .filter_map(|msg| match msg {
                Message::System { .. } => None,
                Message::User { content } => Some(json!({
                    "role": "user",
                    "parts": [{ "text": content }]
                })),
                Message::Assistant { content } => Some(json!({
                    "role": "model",
                    "parts": [{ "text": content }]
                })),
            })
            .collect::<Vec<_>>();

        // Build generation config
        let mut generation_config = json!({
            "temperature": props.temperature,
            "maxOutputTokens": props.max_tokens
        });

        if props.json_mode {
            generation_config["responseMimeType"] = json!("application/json");
        } else {
            generation_config["responseMimeType"] = json!("text/plain");
        }

        // Construct base body
        let mut body = json!({
            "contents": contents,
            "generationConfig": generation_config
        });

        // Add system instruction if present
        if !system_instruction.is_empty() {
            body["systemInstruction"] = json!({
                "parts": [{ "text": system_instruction }]
            });
        }

        let model: String = props.model.clone().into();

        // Build URL and query parameters
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:{}",
            model,
            if streaming { "streamGenerateContent" } else { "generateContent" }
        );

        let mut request = client.post(&url)
            .query(&[("key", api_key)]);

        if streaming {
            request = request.query(&[("alt", "sse")]);
        }

        Ok(request.json(&body))
    
    }

    fn parse_response(json_text: &str) -> Result<String, Error> {
        let response: ResponseJson = serde_json::from_str(json_text)?;
        response
            .candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .and_then(|p| Some(p.text.clone()))
            .ok_or(Error::Provider("Empty Google response".into()))

    }

    fn stream_eventsource(
        mut event_source: reqwest_eventsource::EventSource, 
        tx: Sender<Result<String, LlmStreamingError>>
    ) {
        #[derive(Debug, serde::Deserialize)]
        struct GeminiResponseChunk {
            candidates: Vec<Candidate>,
        }
        #[derive(Debug, serde::Deserialize)]
        struct Candidate {
            content: Content,
        }
        #[derive(Debug, serde::Deserialize)]
        struct Content {
            parts: Vec<Part>,
        }
        #[derive(Debug, serde::Deserialize)]
        struct Part {
            text: String,
        }

        tokio::spawn(async move {
            while let Some(event_result) = event_source.next().await {
                match event_result {
                    Ok(event) => {
                        if let Event::Message(message) = event {
                            match serde_json::from_str::<GeminiResponseChunk>(&message.data) {
                                Ok(response_chunk) => {
                                    if let Some(text) = response_chunk.candidates
                                        .first()
                                        .and_then(|c| c.content.parts.first())
                                        .map(|p| p.text.clone())
                                    {
                                        if let Err(_) = tx.send(Ok(text)).await {
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

            // Send completion marker
            let _ = tx.send(Ok("[DONE]".to_string())).await;
        });

    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_google_response_parsing() {
        let response = json!({
            "candidates": [{
                "content": {
                    "parts": [{
                        "text": "test response"
                    }]
                }
            }]
        })
        .to_string();

        let result = GeminiProvider::parse_response(&response);
        assert_eq!(result.unwrap(), "test response");
    }
}
