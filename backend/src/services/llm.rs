use std::{str::Utf8Error, time::Duration};

use anyhow::Result;
use reqwest::RequestBuilder;
use reqwest_eventsource::{CannotCloneRequestError, EventSource, RequestBuilderExt};
use tokio::sync::mpsc::Sender;
use tokio_retry::{strategy::{jitter, ExponentialBackoff}, Retry};
use tracing;


use crate::{common::types::models::LlmModel, db::logs::LogRepository, services::types::parse_response::LlmApiRequestProps};
use super::{
    providers::{
        anthropic::AnthropicProvider, 
        deepseek::DeepseekProvider, 
        gemini::GeminiProvider, 
        openai::OpenaiProvider
    }, 
    types::{
        llm_props::LlmProps, parse_response::LlmApiResponseProps, stream::LlmStreamingError
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
    fn build_request(&self) -> Result<(RequestBuilder, String), Error>;
    fn parse_response(json_text: &str) -> Result<LlmApiResponseProps, Error>;
    fn stream_eventsource(event_source: EventSource, tx: Sender<Result<String, LlmStreamingError>>);
    fn create_body(&self) -> serde_json::Value;
}

pub struct Llm {
    props: LlmProps,
    db_log: LogRepository
}

impl Llm {
    pub fn new(props: LlmProps, db_log: LogRepository) -> Self {
        Llm {
            props,
            db_log
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
            Ok(json.to_string())
        }).await
    }

    pub async fn stream(&self, tx: Sender<Result<String, LlmStreamingError>>) -> Result<(), Error> {
        if self.props.json_mode {
            tracing::info!("Json mode not supported in chat mode");
            return Err(Error::UnsupportedMode("Json".to_string(), "Chat".to_string()));
        }

        Ok(self.send_request_stream(tx).await?)
    }

    async fn send_request(&self) -> Result<String, Error> {
        let openai_provider = OpenaiProvider::new(&self.props, false);
        let anthropic_provider = AnthropicProvider::new(&self.props, false);
        let gemini_provider = GeminiProvider::new(&self.props, false);
        let deepseek_provider = DeepseekProvider::new(&self.props, false);

        let (request_builder, body) = match &self.props.model {
            LlmModel::OpenAi(_) => openai_provider.build_request(),
            LlmModel::Anthropic(_) => anthropic_provider.build_request(),
            LlmModel::Gemini(_) => gemini_provider.build_request(),
            LlmModel::Deepseek(_) => deepseek_provider.build_request(),
        }?;

        // Convert RequestBuilder to Request to capture details
        let request = request_builder.build()?;
        let method = request.method().to_string();
        let url = request.url().to_string();
        let headers = request.headers().clone();

        // Send the request
        let response = reqwest::Client::new().execute(request).await?;
        let status = response.status();
        let text = response.text().await?;

        // Create request props with captured details
        let request = LlmApiRequestProps::new(
            status.as_u16(),
            body,
            method,
            url,
            headers
        );

        if !status.is_success() {
            return Err(Error::Http(status));
        }

        let mut response = match &self.props.model {
            LlmModel::OpenAi(_) => OpenaiProvider::parse_response(&text)?,
            LlmModel::Anthropic(_) => AnthropicProvider::parse_response(&text)?,
            LlmModel::Gemini(_) => GeminiProvider::parse_response(&text)?,
            LlmModel::Deepseek(_) => DeepseekProvider::parse_response(&text)?
        };

        todo!()
    }

    async fn send_request_stream(
        &self,
        tx: Sender<Result<String, LlmStreamingError>>
    ) -> Result<(), Error> {
        let openai_provider = OpenaiProvider::new(&self.props, true);
        let anthropic_provider = AnthropicProvider::new(&self.props, true);
        let gemini_provider = GeminiProvider::new(&self.props, true);
        let deepseek_provider = DeepseekProvider::new(&self.props, true);

        let (request, body) = match &self.props.model {
            LlmModel::OpenAi(_) => openai_provider.build_request(),
            LlmModel::Anthropic(_) => anthropic_provider.build_request(),
            LlmModel::Gemini(_) => gemini_provider.build_request(),
            LlmModel::Deepseek(_) => deepseek_provider.build_request()
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
    use tokio::sync::mpsc;

    async fn create_test_props(model: LlmModel) -> LlmProps {
        LlmProps {
            model,
            temperature: 0.5,
            max_tokens: 100,
            json_mode: false,
            messages: Message::defaults(),
            prompt_id: 1,
            model_id: 1,
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_openai_integration() {
        dotenv().ok();
        let props = create_test_props(LlmModel::OpenAi(OpenAiModel::Gpt4oMini202407)).await;
        let logs = LogRepository::in_memory().await.unwrap();
        let llm = Llm::new(props, logs);

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
            ],
            prompt_id: 1,
            model_id: 1,
        };


        let logs = LogRepository::in_memory().await.unwrap();
        let llm = Llm::new(props, logs);
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
            ],
            prompt_id: 1,
            model_id: 1,
        };

        let logs = LogRepository::in_memory().await.unwrap();
        let llm = Llm::new(props, logs);

        // Test text response - Anthropic doesn't support JSON mode
        let text = llm.text().await.unwrap();
        assert!(text.contains("Hello"));
    }

    #[tokio::test]
    #[ignore]
    async fn test_gemini_integration() {
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
            ],
            prompt_id: 1,
            model_id: 1,
        };

        let logs = LogRepository::in_memory().await.unwrap();
        let llm = Llm::new(props, logs);
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
            ],
            prompt_id: 1,
            model_id: 1,
        };

        let logs = LogRepository::in_memory().await.unwrap();
        let llm = Llm::new(props, logs);
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
        let logs = LogRepository::in_memory().await.unwrap();
        let llm = Llm::new(props, logs);
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
            ],
            prompt_id: 1,
            model_id: 1,
        };

        let logs = LogRepository::in_memory().await.unwrap();
        let llm = Llm::new(props, logs);
        let response = llm.json().await.unwrap();
        let json: TestResponse = serde_json::from_str(&response).unwrap();
        assert_eq!(json.message, "Hello in JSON");
    }

    async fn create_stream_test_props(model: LlmModel) -> LlmProps {
        LlmProps {
            model,
            temperature: 0.5,
            max_tokens: 100,
            json_mode: false,
            messages: vec![
                Message::System {
                    content: "You are a helpful assistant".to_string(),
                },
                Message::User {
                    content: "Say 'Hello, world!' in a few words".to_string(),
                },
            ],
            prompt_id: 1,
            model_id: 1,
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_openai_stream() {
        dotenv().ok();
        let props = create_stream_test_props(LlmModel::OpenAi(OpenAiModel::Gpt4oMini202407)).await;
        let logs = LogRepository::in_memory().await.unwrap();
        let llm = Llm::new(props, logs);
        let (tx, mut rx) = mpsc::channel(10);

        llm.stream(tx).await.expect("Streaming failed");

        let mut received_chunks = Vec::new();
        while let Some(result) = rx.recv().await {
            let chunk = result.expect("Failed to receive chunk");
            received_chunks.push(chunk);
        }

        assert!(!received_chunks.is_empty());
        let combined = received_chunks.join("");
        assert!(combined.contains("Hello"));
    }

    #[tokio::test]
    #[ignore]
    async fn test_anthropic_stream() {
        dotenv().ok();
        let props =
            create_stream_test_props(LlmModel::Anthropic(AnthropicModel::Claude35Haiku20241022))
                .await;
        let logs = LogRepository::in_memory().await.unwrap();
        let llm = Llm::new(props, logs);
        let (tx, mut rx) = mpsc::channel(10);

        llm.stream(tx).await.expect("Streaming failed");

        let mut received_chunks = Vec::new();
        while let Some(result) = rx.recv().await {
            let chunk = result.expect("Failed to receive chunk");
            received_chunks.push(chunk);
        }

        assert!(!received_chunks.is_empty());
        let combined = received_chunks.join("");
        assert!(combined.contains("Hello"));
    }

    #[tokio::test]
    #[ignore]
    async fn test_gemini_stream() {
        dotenv().ok();
        let props =
            create_stream_test_props(LlmModel::Gemini(GeminiModel::Gemini15Flash)).await;
        let logs = LogRepository::in_memory().await.unwrap();
        let llm = Llm::new(props, logs);
        let (tx, mut rx) = mpsc::channel(10);

        llm.stream(tx).await.expect("Streaming failed");

        let mut received_chunks = Vec::new();
        while let Some(result) = rx.recv().await {
            let chunk = result.expect("Failed to receive chunk");
            received_chunks.push(chunk);
        }

        assert!(!received_chunks.is_empty());
        let combined = received_chunks.join("");
        assert!(combined.contains("Hello"));
    }

    #[tokio::test]
    #[ignore]
    async fn test_deepseek_stream() {
        dotenv().ok();
        let props =
            create_stream_test_props(LlmModel::Deepseek(DeepseekModel::DeepseekChat)).await;
        let logs = LogRepository::in_memory().await.unwrap();
        let llm = Llm::new(props, logs);
        let (tx, mut rx) = mpsc::channel(10);

        llm.stream(tx).await.expect("Streaming failed");

        let mut received_chunks = Vec::new();
        while let Some(result) = rx.recv().await {
            let chunk = result.expect("Failed to receive chunk");
            received_chunks.push(chunk);
        }

        assert!(!received_chunks.is_empty());
        let combined = received_chunks.join("");
        assert!(combined.contains("Hello"));
    }

    async fn create_multi_turn_test_props(model: LlmModel) -> LlmProps {
        LlmProps {
            model,
            temperature: 0.5,
            max_tokens: 100,
            json_mode: false,
            messages: vec![
                Message::System {
                    content: "You are a math tutor who loves to help with algebra".to_string(),
                },
                Message::User {
                    content: "What is 2x + 3 = 7?".to_string(),
                },
                Message::Assistant {
                    content: "Let me help solve this. First, subtract 3 from both sides: 2x = 4. Then divide both sides by 2: x = 2".to_string(),
                },
                Message::User {
                    content: "Great! Now what is x + 5?".to_string(),
                },
            ],
            prompt_id: 1,
            model_id: 1,
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_multi_turn_conversation() {
        dotenv().ok();
        
        // Test each model implementation
        let models = vec![
            LlmModel::OpenAi(OpenAiModel::Gpt4oMini202407),
            LlmModel::Anthropic(AnthropicModel::Claude35Haiku20241022),
            LlmModel::Gemini(GeminiModel::Gemini15Flash),
            LlmModel::Deepseek(DeepseekModel::DeepseekChat),
        ];

        for model in models {
            let props = create_multi_turn_test_props(model.clone()).await;
            let logs = LogRepository::in_memory().await.unwrap();
            let llm = Llm::new(props, logs);

            // The response should continue the conversation naturally
            let response = llm.text().await.unwrap();
            assert!(response.contains("7"), "Response should solve x + 5 = 7 for model {:?}", model);
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_multi_turn_stream() {
        dotenv().ok();
        
        let models = vec![
            LlmModel::OpenAi(OpenAiModel::Gpt4oMini202407),
            LlmModel::Anthropic(AnthropicModel::Claude35Haiku20241022),
            LlmModel::Gemini(GeminiModel::Gemini15Flash),
            LlmModel::Deepseek(DeepseekModel::DeepseekChat),
        ];

        for model in models {
            let props = create_multi_turn_test_props(model.clone()).await;
            let logs = LogRepository::in_memory().await.unwrap();
            let llm = Llm::new(props, logs);
            let (tx, mut rx) = mpsc::channel(10);

            llm.stream(tx).await.expect("Streaming failed");

            let mut received_chunks = Vec::new();
            while let Some(result) = rx.recv().await {
                let chunk = result.expect("Failed to receive chunk");
                received_chunks.push(chunk);
            }

            assert!(!received_chunks.is_empty(), "Should receive chunks for model {:?}", model);
            let combined = received_chunks.join("");
            assert!(combined.contains("7"), 
                "Streamed response should solve x + 5 = 7 for model {:?}", model);
        }
    }

}
