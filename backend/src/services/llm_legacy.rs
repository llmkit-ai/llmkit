use dotenv::dotenv;
use futures_util::StreamExt;

use serde::de::DeserializeOwned;
use serde_json::{json, Value};

use tera::{Context as TeraContext, Tera};
use tokio::sync::mpsc::Sender;
use tokio_retry::{
    strategy::{jitter, ExponentialBackoff},
    Retry,
};

use reqwest_eventsource::{CannotCloneRequestError, Event, EventSource, RequestBuilderExt};

use std::{str::Utf8Error, time::Duration};

use super::types::llm_props::LlmProps;
use crate::{common::types::models::LlmModel, services::types::stream::LlmStreamingError};

#[derive(Debug, thiserror::Error)]
pub enum Error {
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
    #[error("MPSC Sender failed to send message in channel: {0}")]
    MpscSender(#[from] tokio::sync::mpsc::error::SendError<std::string::String>),
    #[error("Invalid UTF8 in chunk: {0}")]
    Utf8Error(#[from] Utf8Error),
    #[error("Eventsource cannot clone request: {0}")]
    EventSourceError(#[from] CannotCloneRequestError),
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ResponseFormat {
    Text,
    Json,
}

pub struct Llm {
    props: LlmProps,
    client: reqwest::Client,
    tera: Tera,
}

impl Llm {
    pub fn new(props: LlmProps) -> Result<Self, Error> {
        dotenv().ok();

        let mut tera = Tera::default();
        tera.add_raw_template("prompt", &props.prompt)
            .map_err(|e| Error::Template(e))?;

        Ok(Self {
            props,
            client: reqwest::Client::new(),
            tera,
        })
    }

    pub async fn text(&self) -> Result<String, Error> {
        let retry_strategy = self.retry_strategy();
        Retry::spawn(retry_strategy, || self.send_request(ResponseFormat::Text)).await
    }

    pub async fn json<T: DeserializeOwned>(&self) -> Result<T, Error> {
        let retry_strategy = self.retry_strategy();
        Retry::spawn(retry_strategy, || async {
            let text = self.send_request(ResponseFormat::Json).await?;
            serde_json::from_str(&text).map_err(Into::into)
        })
        .await
    }

    pub async fn stream(&self, tx: Sender<Result<String, LlmStreamingError>>) -> Result<(), Error> {
        self.send_request_stream(ResponseFormat::Text, tx).await?;
        Ok(())
    }

    async fn send_request(&self, format: ResponseFormat) -> Result<String, Error> {
        let mut tera_ctx = TeraContext::new();
        if let Value::Object(context) = &self.props.context {
            for (k, v) in context {
                tera_ctx.insert(k, v);
            }
        }

        let rendered_prompt = self.tera.render("prompt", &tera_ctx)?;
        let messages = parse_rendered_prompt(&rendered_prompt)?;

        let model_name: String = self.props.model.clone().into();
        let request = match &self.props.model {
            LlmModel::OpenAi(_) => self.build_openai_request(&model_name, format, &messages, false),
            LlmModel::Anthropic(_) => self.build_anthropic_request(&model_name, format, &messages, false),
            LlmModel::Gemini(_) => self.build_google_request(&model_name, format, &messages, false),
            LlmModel::Deepseek(_) => self.build_deepseek_request(&model_name, format, &messages, false)
        }?;

        let response = request.send().await?;
        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            return Err(Error::Http(status));
        }

        match &self.props.model {
            LlmModel::OpenAi(_) => Self::parse_openai_response(&text),
            LlmModel::Anthropic(_) => Self::parse_anthropic_response(&text),
            LlmModel::Gemini(_) => Self::parse_google_response(&text),
            LlmModel::Deepseek(_) => Self::parse_deepseek_response(&text),
        }
    }

    async fn send_request_stream(
        &self,
        format: ResponseFormat,
        tx: Sender<Result<String, LlmStreamingError>>
    ) -> Result<(), Error> {
        let mut tera_ctx = TeraContext::new();
        if let Value::Object(context) = &self.props.context {
            for (k, v) in context {
                tera_ctx.insert(k, v);
            }
        }

        let rendered_prompt = self.tera.render("prompt", &tera_ctx)?;
        let messages = parse_rendered_prompt(&rendered_prompt)?;
        let model_name: String = self.props.model.clone().into();
        let request = match &self.props.model {
            LlmModel::OpenAi(_) => self.build_openai_request(&model_name, format, &messages, true),
            LlmModel::Anthropic(_) => self.build_anthropic_request(&model_name, format, &messages, true),
            LlmModel::Gemini(_) => self.build_google_request(&model_name, format, &messages, true),
            LlmModel::Deepseek(_) => self.build_deepseek_request(&model_name, format, &messages, true)
        }?;

        let event_source = request.eventsource()?;

        match &self.props.model {
            LlmModel::OpenAi(_) => Self::stream_eventsource_openai(event_source, tx).await,
            LlmModel::Anthropic(_) => Self::stream_eventsource_anthropic(event_source, tx).await,
            LlmModel::Gemini(_) => Self::stream_eventsource_gemini(event_source, tx).await,
            LlmModel::Deepseek(_) => Self::stream_eventsource_openai(event_source, tx).await
        }

        Ok(())
    }

    fn build_openai_request(
        &self,
        model: &str,
        format: ResponseFormat,
        messages: &[Message],
        streaming: bool,
    ) -> Result<reqwest::RequestBuilder, Error> {
        let api_key = std::env::var("OPENAI_API_KEY").map_err(|_| Error::Auth)?;
        let messages_json: Vec<_> = messages
            .iter()
            .map(|msg| json!({ "role": msg.role, "content": msg.content }))
            .collect();

        let mut body = serde_json::json!({
            "model": model,
            "messages": messages_json,
            "stream": streaming
        });

        if format == ResponseFormat::Json {
            body["response_format"] = serde_json::json!({ "type": "json_object" });
        }

        body["temperature"] = self.props.temperature.into();
        body["max_completion_tokens"] = self.props.max_tokens.into();

        Ok(self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&body))
    }

    fn build_anthropic_request(
        &self,
        model: &str,
        _format: ResponseFormat,
        messages: &[Message],
        streaming: bool,
    ) -> Result<reqwest::RequestBuilder, Error> {
        let api_key = std::env::var("ANTHROPIC_API_KEY").map_err(|_| Error::Auth)?;

        // Extract system messages and combine them
        let system_messages: Vec<String> = messages
            .iter()
            .filter(|msg| msg.role == "system")
            .map(|msg| msg.content.clone())
            .collect();
        let system = system_messages.join("\n\n");

        // Convert remaining messages to Anthropic format
        let messages_json: Vec<_> = messages
            .iter()
            .filter(|msg| msg.role != "system")
            .map(|msg| {
                json!({
                    "role": msg.role,
                    "content": msg.content
                })
            })
            .collect();

        let mut body = json!({
            "model": model,
            "messages": messages_json,
            "system": system,
            "stream": streaming
        });

        // Add optional parameters
        body["temperature"] = self.props.temperature.into();
        body["max_tokens"] = self.props.max_tokens.into();

        Ok(self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body))
    }

    fn build_google_request(
        &self,
        model: &str,
        format: ResponseFormat,
        messages: &[Message],
        streaming: bool,
    ) -> Result<reqwest::RequestBuilder, Error> {
        let api_key = std::env::var("GOOGLE_API_KEY").map_err(|_| Error::Auth)?;

        // Extract system messages and combine them
        let system_messages: Vec<String> = messages
            .iter()
            .filter(|msg| msg.role == "system")
            .map(|msg| msg.content.clone())
            .collect();
        let system_instruction = system_messages.join("\n\n");

        // Prepare generation config
        let mut generation_config = json!({
            "temperature": self.props.temperature,
            "maxOutputTokens": self.props.max_tokens
        });

        if format == ResponseFormat::Json {
            generation_config["responseMimeType"] = json!("application/json");
        } else {
            generation_config["responseMimeType"] = json!("text/plain");
        }

        // Build main request body
        let mut body = json!({
            "contents": [{
                "parts": messages
                    .iter()
                    .filter(|msg| msg.role == "user")
                    .map(|msg| json!({ "text": msg.content }))
                    .collect::<Vec<_>>()
            }],
            "generationConfig": generation_config
        });

        // Add system instruction if present
        if !system_instruction.is_empty() {
            body["systemInstruction"] = json!({
                "parts": [{ "text": system_instruction }]
            });
        }

        // Handle additional parts from context
        if let Some(parts) = self.props.context.get("parts") {
            if let Some(content_array) = body["contents"][0]["parts"].as_array_mut() {
                for part in parts.as_array().unwrap_or(&vec![]) {
                    content_array.push(part.clone());
                }
            }
        }

        match streaming {
            true => {
                let url = &format!("https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent", model);
                Ok(self
                    .client
                    .post(url)
                    .query(&[("key", api_key), ("alt", "sse".to_string())])
                    .json(&body))
            }
            false => {
                let url = &format!(
                    "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
                    model
                );
                Ok(self.client.post(url).query(&[("key", api_key)]).json(&body))
            }
        }
    }

    fn build_deepseek_request(
        &self,
        model: &str,
        format: ResponseFormat,
        messages: &[Message],
        streaming: bool,
    ) -> Result<reqwest::RequestBuilder, Error> {
        let api_key = std::env::var("DEEPSEEK_API_KEY").map_err(|_| Error::Auth)?;

        let messages_json: Vec<_> = messages
            .iter()
            .map(|msg| json!({ "role": msg.role, "content": msg.content }))
            .collect();

        let mut body = serde_json::json!({
            "model": model,
            "messages": messages_json,
            "stream": streaming
        });

        body["temperature"] = self.props.temperature.into();
        body["max_tokens"] = self.props.max_tokens.into();

        if format == ResponseFormat::Json {
            body["response_format"] = serde_json::json!({ "type": "json_object" });
        }

        Ok(self
            .client
            .post("https://api.deepseek.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&body))
    }

    // Response parsing functions remain the same as previous implementation
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

    fn retry_strategy(&self) -> impl Iterator<Item = Duration> {
        ExponentialBackoff::from_millis(100)
            .max_delay(Duration::from_secs(100))
            .map(jitter)
            .take(10)
    }

    async fn stream_eventsource_openai(mut event_source: EventSource, tx: Sender<Result<String, LlmStreamingError>>) {
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

    async fn stream_eventsource_anthropic(mut event_source: EventSource, tx: Sender<Result<String, LlmStreamingError>>) {
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

    async fn stream_eventsource_gemini(mut event_source: EventSource, tx: Sender<Result<String, LlmStreamingError>>) {
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

#[derive(Debug)]
struct Message {
    role: String,
    content: String,
}

fn parse_rendered_prompt(rendered_prompt: &str) -> Result<Vec<Message>, Error> {
    let mut messages = Vec::new();
    let mut current_role = None;
    let mut current_content = String::new();

    for line in rendered_prompt.lines() {
        let line = line.trim();
        if let Some(role) = parse_role_line(line) {
            if let Some(prev_role) = current_role.take() {
                messages.push(Message {
                    role: prev_role,
                    content: current_content.trim().to_string(),
                });
                current_content.clear();
            }
            current_role = Some(role);
        } else if !line.is_empty() {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }

    if let Some(role) = current_role {
        messages.push(Message {
            role,
            content: current_content.trim().to_string(),
        });
    }

    if messages.is_empty() {
        Err(Error::NoRoleSections)
    } else {
        Ok(messages)
    }
}

fn parse_role_line(line: &str) -> Option<String> {
    const PREFIX: &str = "<!-- role:";
    const SUFFIX: &str = "-->";

    if line.starts_with(PREFIX) && line.ends_with(SUFFIX) {
        let role = line[PREFIX.len()..line.len() - SUFFIX.len()]
            .trim()
            .to_string();
        match role.as_str() {
            "system" | "user" | "assistant" => Some(role),
            _ => None,
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::types::models::{
        AnthropicModel, DeepseekModel, GeminiModel, LlmModel, OpenAiModel,
    };
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

        let result = Llm::parse_openai_response(&response);
        assert_eq!(result.unwrap(), "test response");
    }

    #[test]
    fn test_anthropic_response_parsing() {
        let response = json!({
            "content": [{
                "text": "test response"
            }]
        })
        .to_string();

        let result = Llm::parse_anthropic_response(&response);
        assert_eq!(result.unwrap(), "test response");
    }

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

        let result = Llm::parse_google_response(&response);
        assert_eq!(result.unwrap(), "test response");
    }

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

        let result = Llm::parse_deepseek_response(&response);
        assert_eq!(result.unwrap(), "test response");
    }

    async fn create_test_props(model: LlmModel) -> LlmProps {
        LlmProps {
            model,
            prompt: r#"<!-- role:system -->
                You are a helpful assistant
                <!-- role:user -->
                Hello, {{ name }}!"#
                .to_string(),
            context: json!({
                "name": "World",
            }),
            temperature: 0.5,
            max_tokens: 100,
            json_mode: false,
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_openai_integration() {
        dotenv().ok();
        let props = create_test_props(LlmModel::OpenAi(OpenAiModel::Gpt4oMini202407)).await;
        let llm = Llm::new(props).unwrap();

        // Test text response
        let text = llm.text().await.unwrap();
        assert!(text.contains("Hello"));

        // Test JSON response
        #[derive(serde::Deserialize)]
        struct TestResponse {
            message: String,
        }

        let props = LlmProps {
            prompt: r#"<!-- role:system -->
                You must respond with valid JSON only
                <!-- role:user -->
                Return a JSON object with a 'message' field containing 'Hello in JSON'. Context: {{ message }}"#.to_string(),
            context: json!({
                "message": "Please respond with JSON",
                "response_format": {"type": "json_object"}
            }),
            ..create_test_props(LlmModel::OpenAi(OpenAiModel::Gpt4oMini202407)).await
        };

        let llm = Llm::new(props).unwrap();
        let response: TestResponse = llm.json().await.unwrap();
        assert_eq!(response.message, "Hello in JSON");
    }

    #[tokio::test]
    #[ignore]
    async fn test_anthropic_integration() {
        dotenv().ok();
        let props =
            create_test_props(LlmModel::Anthropic(AnthropicModel::Claude35Haiku20241022)).await;
        let llm = Llm::new(props).unwrap();

        let text = llm.text().await.unwrap();
        assert!(text.contains("Hello"));
    }

    #[tokio::test]
    #[ignore]
    async fn test_google_integration() {
        dotenv().ok();

        // Test text response
        let props = create_test_props(LlmModel::Gemini(GeminiModel::Gemini15Flash)).await;
        let llm = Llm::new(props).unwrap();
        let text = llm.text().await.unwrap();
        assert!(text.contains("Hello"));

        // Test JSON response
        #[derive(serde::Deserialize)]
        struct TestResponse {
            message: String,
        }

        let props = LlmProps {
            prompt: r#"<!-- role:system -->
                You must respond with valid JSON only
                <!-- role:user -->
                Return a JSON object with a 'message' field containing 'Hello in JSON'. Context: {{ message }}"#.to_string(),
            context: json!({
                "message": "Please respond with JSON",
                "responseMimeType": "application/json"
            }),
            ..create_test_props(LlmModel::Gemini(GeminiModel::Gemini15Flash)).await
        };

        let llm = Llm::new(props).unwrap();
        let response: TestResponse = llm.json().await.unwrap();
        assert_eq!(response.message, "Hello in JSON");
    }

    #[tokio::test]
    #[ignore]
    async fn test_deepseek_integration() {
        dotenv().ok();

        // Test text response
        let props = create_test_props(LlmModel::Deepseek(DeepseekModel::DeepseekChat)).await;
        let llm = Llm::new(props).unwrap();
        let text = llm.text().await.unwrap();
        assert!(text.contains("Hello"));

        // Test JSON response
        #[derive(serde::Deserialize)]
        struct TestResponse {
            #[serde(rename = "content")]
            message: String,
        }

        let props = LlmProps {
            prompt: r#"<!-- role:system -->
                Respond with JSON containing a 'content' field
                <!-- role:user -->
                Return JSON with format: {"content": "<message>"}. Context: {{ message }}"#
                .to_string(),
            context: json!({
                "message": "Hello in JSON",
                "response_format": {"type": "json_object"}
            }),
            ..create_test_props(LlmModel::Deepseek(DeepseekModel::DeepseekChat)).await
        };

        let llm = Llm::new(props).unwrap();
        let response: TestResponse = llm.json().await.unwrap();
        assert_eq!(response.message, "Hello in JSON");
    }

    #[tokio::test]
    async fn test_retry_logic() {
        let props = LlmProps {
            model: LlmModel::OpenAi(OpenAiModel::Gpt4oMini202407),
            prompt: "test".to_string(),
            context: json!({}),
            temperature: 0.5,
            max_tokens: 100,
            json_mode: false,
        };
        let llm = Llm::new(props).unwrap();

        let strategy = llm.retry_strategy();
        assert_eq!(strategy.count(), 3);
    }
}
