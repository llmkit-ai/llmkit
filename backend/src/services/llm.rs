use std::{str::Utf8Error, time::Duration};

use anyhow::Result;
use reqwest::RequestBuilder;
use reqwest_eventsource::{CannotCloneRequestError, EventSource, RequestBuilderExt};
use tokio::sync::mpsc::Sender;
use tokio_retry::{strategy::{jitter, ExponentialBackoff}, Retry};
use tracing;


use crate::common::types::models::LlmModel;
use super::{
    providers::{
        anthropic::AnthropicProvider, 
        deepseek::DeepseekProvider, 
        gemini::GeminiProvider, 
        openai::OpenaiProvider
    }, 
    types::{
        llm_props::LlmProps, 
        stream::LlmStreamingError
    }
};


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
    #[error("{0} not supported in {1}")]
    UnsupportedMode(String, String),
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

pub trait LlmProvider {
    fn build_request(props: &LlmProps, streaming: bool) -> Result<RequestBuilder, Error>;
    fn parse_response(json_text: &str) -> Result<String, Error>;
    fn stream_eventsource(event_source: EventSource, tx: Sender<Result<String, LlmStreamingError>>);
}

pub struct Llm {
    props: LlmProps
}

impl Llm {
    pub fn new(props: LlmProps) -> Self {
        Llm {
            props
        } 
    }

    fn retry_strategy(&self) -> impl Iterator<Item = Duration> {
        ExponentialBackoff::from_millis(100)
            .max_delay(Duration::from_secs(100))
            .map(jitter)
            .take(1) 
    }

    pub async fn text(&self) -> Result<String, Error> {
        let retry_strategy = self.retry_strategy();
        Retry::spawn(retry_strategy, || self.send_request()).await
    }

    pub async fn json(&self) -> Result<String, Error> {
        let retry_strategy = self.retry_strategy();
        Retry::spawn(retry_strategy, || async {
            let text = self.send_request().await?;
            // Since this is not a client library and will be invoked via API
            // we can't strongly enforce the shape of the JSON, therefore we just
            // need to make sure it is a valid JSON (hence Value) and then convert
            // it back into text and be on our way
            let json: serde_json::Value = serde_json::from_str(&text)?;
            Ok(serde_json::to_string(&json).expect("Failed to convert json back into string"))
        }).await
    }

    pub async fn stream(&self, tx: Sender<Result<String, LlmStreamingError>>) -> Result<(), Error> {
        if self.props.json_mode {
            tracing::info!("Json mode not supported in chat mode");
            return Err(Error::UnsupportedMode("Json".to_string(), "Chat".to_string()));
        }

        self.send_request_stream(tx).await?;
        todo!()
    }

    async fn send_request(&self) -> Result<String, Error> {
        let request = match &self.props.model {
            LlmModel::OpenAi(_) => OpenaiProvider::build_request(&self.props, false),
            LlmModel::Anthropic(_) => AnthropicProvider::build_request(&self.props, false),
            LlmModel::Gemini(_) => GeminiProvider::build_request(&self.props, false),
            LlmModel::Deepseek(_) => DeepseekProvider::build_request(&self.props, false)
        }?;

        let response = request.send().await?;
        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            return Err(Error::Http(status));
        }

        match &self.props.model {
            LlmModel::OpenAi(_) => OpenaiProvider::parse_response(&text),
            LlmModel::Anthropic(_) => AnthropicProvider::parse_response(&text),
            LlmModel::Gemini(_) => GeminiProvider::parse_response(&text),
            LlmModel::Deepseek(_) => DeepseekProvider::parse_response(&text)
        }
    }

    async fn send_request_stream(
        &self,
        tx: Sender<Result<String, LlmStreamingError>>
    ) -> Result<(), Error> {
        let request = match &self.props.model {
            LlmModel::OpenAi(_) => OpenaiProvider::build_request(&self.props, true),
            LlmModel::Anthropic(_) => AnthropicProvider::build_request(&self.props, true),
            LlmModel::Gemini(_) => GeminiProvider::build_request(&self.props, true),
            LlmModel::Deepseek(_) => DeepseekProvider::build_request(&self.props, true)
        }?;

        let event_source = request.eventsource()?;

        match &self.props.model {
            LlmModel::OpenAi(_) => OpenaiProvider::stream_eventsource(event_source, tx),
            LlmModel::Anthropic(_) => AnthropicProvider::stream_eventsource(event_source, tx),
            LlmModel::Gemini(_) => GeminiProvider::stream_eventsource(event_source, tx),
            LlmModel::Deepseek(_) => DeepseekProvider::stream_eventsource(event_source, tx)
        }

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::{common::types::models::{
        AnthropicModel, DeepseekModel, GeminiModel, LlmModel, OpenAiModel,
    }, services::types::message::Message};
    use dotenv::dotenv;

    async fn create_test_props(model: LlmModel) -> LlmProps {
        LlmProps {
            model,
            temperature: 0.5,
            max_tokens: 100,
            json_mode: false,
            messages: Message::defaults()
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_openai_integration() {
        dotenv().ok();
        let props = create_test_props(LlmModel::OpenAi(OpenAiModel::Gpt4oMini202407)).await;
        let llm = Llm::new(props);

        // Test text response
        let text = llm.text().await.unwrap();
        assert!(text.contains("Hello"));

        // Test JSON response
        #[derive(serde::Deserialize)]
        struct TestResponse {
            message: String,
        }

        let props = LlmProps {
            model: LlmModel::OpenAi(OpenAiModel::Gpt4oMini202407),
            temperature: 0.5,
            max_tokens: 100,
            json_mode: true,
            messages: vec![
                Message::System { content: "You must respond with valid JSON onlyl".to_string() },
                Message::User { content: "Return a JSON object with a 'message' field containing 'Hello in JSON'".to_string() },
            ]
        };


        let llm = Llm::new(props);
        let response = llm.json().await.unwrap();
        let json: TestResponse = serde_json::from_str(&response).unwrap();
        assert_eq!(json.message, "Hello in JSON");
    }

    #[tokio::test]
    #[ignore]
    async fn test_anthropic_integration() {
        dotenv().ok();
        let props = LlmProps {
            model: LlmModel::Anthropic(AnthropicModel::Claude35Haiku20241022),
            temperature: 0.5,
            max_tokens: 100,
            json_mode: false,
            messages: vec![
                Message::System { content: "You are a friendly assistance".to_string() },
                Message::User { content: "You return a message saying 'Hello'".to_string() },
            ]
        };
        let llm = Llm::new(props);

        // Test text response - Anthropic doesn't support JSON mode
        let text = llm.text().await.unwrap();
        assert!(text.contains("Hello"));
    }

    #[tokio::test]
    #[ignore]
    async fn test_google_integration() {
        dotenv().ok();

        // Test text response
        let props = LlmProps {
            model: LlmModel::Anthropic(AnthropicModel::Claude35Haiku20241022),
            temperature: 0.5,
            max_tokens: 100,
            json_mode: false,
            messages: vec![
                Message::System { content: "You are a friendly assistance".to_string() },
                Message::User { content: "You return a message saying 'Hello'".to_string() },
            ]
        };
        let llm = Llm::new(props);
        let text = llm.text().await.unwrap();
        assert!(text.contains("Hello"));

        // Test JSON response
        #[derive(serde::Deserialize)]
        struct TestResponse {
            message: String,
        }

        let props = LlmProps {
            model: LlmModel::Gemini(GeminiModel::Gemini15Flash),
            temperature: 0.5,
            max_tokens: 100,
            json_mode: true,
            messages: vec![
                Message::System { 
                    content: "You must respond with valid JSON only".to_string() 
                },
                Message::User { 
                    content: "Return a JSON object with a 'message' field containing 'Hello in JSON'".to_string() 
                },
            ]
        };

        let llm = Llm::new(props);
        let response = llm.json().await.unwrap();
        let json: TestResponse = serde_json::from_str(&response).unwrap();
        assert_eq!(json.message, "Hello in JSON");
    }

    #[tokio::test]
    #[ignore]
    async fn test_deepseek_integration() {
        dotenv().ok();

        // Test text response
        let props = create_test_props(LlmModel::Deepseek(DeepseekModel::DeepseekChat)).await;
        let llm = Llm::new(props);
        let text = llm.text().await.unwrap();
        assert!(text.contains("Hello"));

        // Test JSON response
        #[derive(serde::Deserialize)]
        struct TestResponse {
            #[serde(rename = "content")]
            message: String,
        }

        let props = LlmProps {
            model: LlmModel::Deepseek(DeepseekModel::DeepseekChat),
            temperature: 0.5,
            max_tokens: 100,
            json_mode: true,
            messages: vec![
                Message::System { 
                    content: "Respond with JSON containing a 'content' field".to_string() 
                },
                Message::User { 
                    content: "Return JSON with format: {\"content\": \"Hello in JSON\"}".to_string() 
                },
            ]
        };

        let llm = Llm::new(props);
        let response = llm.json().await.unwrap();
        let json: TestResponse = serde_json::from_str(&response).unwrap();
        assert_eq!(json.message, "Hello in JSON");
    }

}
