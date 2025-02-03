use std::str::Utf8Error;

use anyhow::Result;
use reqwest_eventsource::CannotCloneRequestError;
use serde_json::json;
use tracing;


use crate::common::types::models::LlmModel;
use super::types::{llm_props::LlmProps, message::Message};

#[derive(Debug, thiserror::Error)]
pub enum Error<'a> {
    #[error("HTTP error: {0}")]
    Http(reqwest::StatusCode),
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Authentication error")]
    Auth,
    #[error("Provider error: {0}")]
    Provider(String),
    #[error("Template error: {0}")]
    Template(#[from] tera::Error),
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("No valid role sections found in prompt")]
    NoRoleSections,
    #[error("Invalid role specified in template: {0}")]
    InvalidRole(String),
    #[error("{0} not supported in {1}")]
    UnsupportedMode(&'a str, &'a str),
    #[error("MPSC Sender failed to send message in channel: {0}")]
    MpscSender(#[from] tokio::sync::mpsc::error::SendError<std::string::String>),
    #[error("Invalid UTF8 in chunk: {0}")]
    Utf8Error(#[from] Utf8Error),
    #[error("Eventsource cannot clone request: {0}")]
    EventSourceError(#[from] CannotCloneRequestError),
    #[error("Missing system message")]
    MissingSystemMessage,
    #[error("Missing system message")]
    MissingUserMessage,
}

pub struct Llm {
    messages: Vec<Message>,
    model: LlmModel,
    model_name: String,
    max_tokens: i64,
    temperature: f64,
    json_mode: bool,
    client: reqwest::Client,
}

impl Llm {
    pub fn new(props: LlmProps) -> Self {
        Llm {
            messages: props.messages,
            model: props.model.clone(),
            model_name: props.model.into(),
            max_tokens: props.max_tokens,
            temperature: props.temperature,
            json_mode: props.json_mode,
            client: reqwest::Client::new(),
        } 
    }

    pub async fn completion(&self) -> Result<()> {
        todo!()
    }

    pub async fn chat(&self) -> Result<(), Error> {
        if self.json_mode {
            tracing::info!("Json mode not supported in chat mode");
            return Err(Error::UnsupportedMode("Json", "Chat"));
        }
        todo!()
    }

    async fn text(&self) -> Result<()> {
        todo!()
    }

    async fn json(&self) -> Result<()> {
        todo!()
    }

    async fn stream(&self) -> Result<(), Error> {
        if self.json_mode {
            tracing::info!("Json mode not supported in chat mode");
            return Err(Error::UnsupportedMode("Json", "Chat"));
        }
        todo!()
    }

    async fn send_request(&self) -> Result<String, Error> {
        let request = match &self.model {
            LlmModel::OpenAi(_) => self.build_openai_request(false),
            LlmModel::Anthropic(_) => self.build_anthropic_request(false),
            LlmModel::Gemini(_) => self.build_google_request(false),
            LlmModel::Deepseek(_) => self.build_deepseek_request(false)
        }?;

        let response = request.send().await?;
        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            return Err(Error::Http(status));
        }

        match &self.model {
            LlmModel::OpenAi(_) => Self::parse_openai_response(&text),
            LlmModel::Anthropic(_) => Self::parse_anthropic_response(&text),
            LlmModel::Gemini(_) => Self::parse_google_response(&text),
            LlmModel::Deepseek(_) => Self::parse_deepseek_response(&text)
        }
    }


    fn build_openai_request(
        &self,
        streaming: bool,
    ) -> Result<reqwest::RequestBuilder, Error> {
        let api_key = std::env::var("OPENAI_API_KEY").map_err(|_| Error::Auth)?;

        // Convert messages to OpenAI format
        let messages = self.messages.iter()
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
            "model": self.model,
            "messages": messages,
            "stream": streaming,
            "temperature": self.temperature,
            "max_tokens": self.max_tokens
        });

        if self.json_mode {
            body["response_format"] = json!({ "type": "json_object" });
        }

        Ok(self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&body))
    }

    fn build_anthropic_request(
        &self,
        streaming: bool,
    ) -> Result<reqwest::RequestBuilder, Error> {
        let api_key = std::env::var("ANTHROPIC_API_KEY").map_err(|_| Error::Auth)?;

        // Extract system messages and combine them
        let system_message = self.messages.iter()
            .find_map(|msg| {
                if let Message::System { content } = msg {
                    Some(content.clone())
                } else {
                    None
                }
            })
            .ok_or(Error::MissingSystemMessage)?;


        // Convert remaining messages to Anthropic format
        let messages = self.messages.iter()
            .find_map(|msg| {
                match msg {
                    Message::System { content: _ } => None,
                    Message::User { content } => {
                        Some(json!({
                            "role": "user",
                            "content": content
                        }))
                    },
                    Message::Assistant { content } => {
                        Some(json!({
                            "role": "assistant",
                            "content": content
                        }))
                    }
                }
            })
            .ok_or(Error::MissingUserMessage)?;

        let body = json!({
            "model": self.model,
            "messages": messages,
            "system": system_message,
            "stream": streaming,
            "temperature": self.temperature,
            "max_tokens": self.max_tokens,
        });

        Ok(self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body))
    }

    fn build_google_request(
        &self,
        streaming: bool,
    ) -> Result<reqwest::RequestBuilder, Error> {
        let api_key = std::env::var("GOOGLE_API_KEY").map_err(|_| Error::Auth)?;

        // Extract and combine system messages
        let system_instruction = self.messages.iter()
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
        let contents = self.messages.iter()
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
            "temperature": self.temperature,
            "maxOutputTokens": self.max_tokens
        });

        if self.json_mode {
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

        // Build URL and query parameters
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:{}",
            self.model_name,
            if streaming { "streamGenerateContent" } else { "generateContent" }
        );

        let mut request = self.client.post(&url)
            .query(&[("key", api_key)]);

        if streaming {
            request = request.query(&[("alt", "sse")]);
        }

        Ok(request.json(&body))
    }

    fn build_deepseek_request(
        &self,
        streaming: bool,
    ) -> Result<reqwest::RequestBuilder, Error> {
        let api_key = std::env::var("DEEPSEEK_API_KEY").map_err(|_| Error::Auth)?;

        let messages = self.messages.iter()
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
            "model": self.model,
            "messages": messages,
            "stream": streaming,
            "temperature": self.temperature,
            "max_tokens": self.max_tokens
        });

        if self.json_mode == true {
            body["response_format"] = serde_json::json!({ "type": "json_object" });
        }

        Ok(self
            .client
            .post("https://api.deepseek.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&body))
    }

    fn parse_openai_response(text: &str) -> Result<String, Error> {
        #[derive(serde::Deserialize)]
        struct Response {
            choices: Vec<Choice>,
        }
        #[derive(serde::Deserialize)]
        struct Choice {
            message: Message,
        }
        #[derive(serde::Deserialize)]
        struct Message {
            content: String,
        }

        let response: Response = serde_json::from_str(text)?;
        response
            .choices
            .first()
            .and_then(|c| Some(c.message.content.clone()))
            .ok_or(Error::Provider("Empty OpenAI response".into()))
    }

    fn parse_anthropic_response(text: &str) -> Result<String, Error> {
        #[derive(serde::Deserialize)]
        struct Response {
            content: Vec<Content>,
        }
        #[derive(serde::Deserialize)]
        struct Content {
            text: String,
        }

        let response: Response = serde_json::from_str(text)?;
        response
            .content
            .first()
            .and_then(|c| Some(c.text.clone()))
            .ok_or(Error::Provider("Empty Anthropic response".into()))
    }

    fn parse_google_response(text: &str) -> Result<String, Error> {
        #[derive(serde::Deserialize)]
        struct Response {
            candidates: Vec<Candidate>,
        }
        #[derive(serde::Deserialize)]
        struct Candidate {
            content: Content,
        }
        #[derive(serde::Deserialize)]
        struct Content {
            parts: Vec<Part>,
        }
        #[derive(serde::Deserialize)]
        struct Part {
            text: String,
        }

        let response: Response = serde_json::from_str(text)?;
        response
            .candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .and_then(|p| Some(p.text.clone()))
            .ok_or(Error::Provider("Empty Google response".into()))
    }

    fn parse_deepseek_response(text: &str) -> Result<String, Error> {
        #[derive(serde::Deserialize)]
        struct Response {
            choices: Vec<Choice>,
        }
        #[derive(serde::Deserialize)]
        struct Choice {
            message: Message,
        }
        #[derive(serde::Deserialize)]
        struct Message {
            content: String,
        }

        let response: Response = serde_json::from_str(text)?;
        response
            .choices
            .first()
            .and_then(|c| Some(c.message.content.clone()))
            .ok_or(Error::Provider("Empty Deepseek response".into()))
    }
}
