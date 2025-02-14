use std::{str::Utf8Error, time::Duration};

use anyhow::Result;
use reqwest::RequestBuilder;
use reqwest_eventsource::{CannotCloneRequestError, EventSource, RequestBuilderExt};
use tokio::{sync::mpsc::Sender, task::JoinError};
use tokio_retry::{strategy::{jitter, ExponentialBackoff}, Retry};
use tracing;


use crate::{common::types::models::LlmApiProvider, db::logs::LogRepository, services::types::parse_response::LlmApiRequestProps};
use super::{
    providers::{
        anthropic::AnthropicProvider, azure::AzureProvider, deepseek::DeepseekProvider, gemini::GeminiProvider, openai::OpenaiProvider
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
    #[error("Missing Usage from chunk")]
    MissingUsage,
    #[error("DB Logging Error: {0}")]
    DbLoggingError(String),
    #[error("JoinError in spawned tokio task: {0}")]
    TokioTaskJoin(#[from] JoinError),
}

pub struct ExecutionResponse {
    pub content: String,
    pub log_id: i64
}

pub trait LlmProvider {
    fn build_request(&self) -> Result<(RequestBuilder, String), Error>;
    fn parse_response(json_text: &str) -> Result<LlmApiResponseProps, Error>;
    async fn stream_eventsource(event_source: EventSource, tx: Sender<Result<String, LlmStreamingError>>) -> Result<LlmApiResponseProps, Error>;
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

    pub async fn text(&self) -> Result<ExecutionResponse, Error> {
        let retry_strategy = self.retry_strategy();
        Retry::spawn(retry_strategy, || self.send_request()).await
    }

    pub async fn json(&self) -> Result<ExecutionResponse, Error> {
        let retry_strategy = self.retry_strategy();
        Retry::spawn(retry_strategy, || async {
            let res = self.send_request().await?;
            // Since this is not a client library and will be invoked via API
            // we can't strongly enforce the shape of the JSON, therefore we just
            // need to make sure it is a valid JSON (hence Value) and then convert
            // it back into text and be on our way
            let _json: serde_json::Value = serde_json::from_str(&res.content)?;
            Ok(res)
        }).await
    }

    pub async fn stream(&self, tx: Sender<Result<String, LlmStreamingError>>) -> Result<ExecutionResponse, Error> {
        if self.props.json_mode {
            tracing::info!("Json mode not supported in chat mode");
            return Err(Error::UnsupportedMode("Json".to_string(), "Chat".to_string()));
        }

        Ok(self.send_request_stream(tx).await?)
    }

    async fn send_request(&self) -> Result<ExecutionResponse, Error> {
        let openai_provider = OpenaiProvider::new(&self.props, false);
        let anthropic_provider = AnthropicProvider::new(&self.props, false);
        let gemini_provider = GeminiProvider::new(&self.props, false);
        let deepseek_provider = DeepseekProvider::new(&self.props, false);
        let azure_provider = AzureProvider::new(&self.props, false);

        let (request_builder, body) = match &self.props.provider {
            LlmApiProvider::OpenAi => openai_provider.build_request(),
            LlmApiProvider::Anthropic => anthropic_provider.build_request(),
            LlmApiProvider::Gemini => gemini_provider.build_request(),
            LlmApiProvider::Deepseek => deepseek_provider.build_request(),
            LlmApiProvider::Azure => azure_provider.build_request()
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
            self.log_request(
                None,
                None,
                None,
                None,
                None,
                &request.body,
            ).await?;

            return Err(Error::Http(status));
        }

        let parsed_response = match &self.props.provider {
            LlmApiProvider::OpenAi => OpenaiProvider::parse_response(&text)?,
            LlmApiProvider::Anthropic => AnthropicProvider::parse_response(&text)?,
            LlmApiProvider::Gemini => GeminiProvider::parse_response(&text)?,
            LlmApiProvider::Deepseek => DeepseekProvider::parse_response(&text)?,
            LlmApiProvider::Azure => AzureProvider::parse_response(&text)?
        };

        let log_id = self.log_request(
            Some(&parsed_response.raw_response),
            Some(request.status as i64),
            parsed_response.input_tokens,
            parsed_response.output_tokens,
            parsed_response.reasoning_tokens,
            &request.body
        ).await?;

        Ok(ExecutionResponse { content: parsed_response.response_content, log_id } )
    }

    async fn send_request_stream(
        &self,
        tx: Sender<Result<String, LlmStreamingError>>
    )  -> Result<ExecutionResponse, Error> {
        let openai_provider = OpenaiProvider::new(&self.props, true);
        let anthropic_provider = AnthropicProvider::new(&self.props, true);
        let gemini_provider = GeminiProvider::new(&self.props, true);
        let deepseek_provider = DeepseekProvider::new(&self.props, true);
        let azure_provider = AzureProvider::new(&self.props, true);

        let (request, body) = match &self.props.provider {
            LlmApiProvider::OpenAi => openai_provider.build_request(),
            LlmApiProvider::Anthropic => anthropic_provider.build_request(),
            LlmApiProvider::Gemini => gemini_provider.build_request(),
            LlmApiProvider::Deepseek => deepseek_provider.build_request(),
            LlmApiProvider::Azure => azure_provider.build_request(),
        }?;

        let event_source = request.eventsource()?;

        let response = match &self.props.provider {
            LlmApiProvider::OpenAi => OpenaiProvider::stream_eventsource(event_source, tx).await?,
            LlmApiProvider::Anthropic => AnthropicProvider::stream_eventsource(event_source, tx).await?,
            LlmApiProvider::Gemini => GeminiProvider::stream_eventsource(event_source, tx).await?,
            LlmApiProvider::Deepseek => DeepseekProvider::stream_eventsource(event_source, tx).await?,
            LlmApiProvider::Azure => AzureProvider::stream_eventsource(event_source, tx).await?,
        };

        let log_id = self.log_request(
            Some(&response.raw_response),
            None,
            response.input_tokens,
            response.output_tokens,
            response.reasoning_tokens,
            &body
        ).await?;


        Ok(ExecutionResponse { content: response.response_content, log_id } )
    }

    /// Logs the request and returns a log ID.
    async fn log_request(
        &self,
        raw_response: Option<&str>,
        status: Option<i64>,
        input_tokens: Option<i64>,
        output_tokens: Option<i64>,
        reasoning_tokens: Option<i64>,
        request_body: &str,
    ) -> Result<i64, Error> {
        self.db_log
            .create_log(
                Some(self.props.prompt_id),
                self.props.model_id,
                raw_response,
                status,
                input_tokens,
                output_tokens,
                reasoning_tokens,
                Some(request_body),
            )
            .await
            .map_err(|e| Error::DbLoggingError(e.to_string()))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        common::types::models::LlmApiProvider,
        db::prompts::PromptRepository,
        services::types::message::Message,
    };
    use dotenv::dotenv;
    use tokio::sync::mpsc;

    async fn create_test_props(provider: LlmApiProvider, model_name: String) -> LlmProps {
        LlmProps {
            provider,
            model_name,
            temperature: 0.5,
            max_tokens: 100,
            json_mode: false,
            messages: Message::defaults(),
            prompt_id: 1,
            model_id: 1,
        }
    }

    async fn create_shared_in_memory_pool() -> Result<sqlx::SqlitePool, sqlx::Error> {
        // Use the shared cache so all connections share the same in-memory database.
        let pool = sqlx::SqlitePool::connect("sqlite::memory:?cache=shared").await?;
        // Run migrations to set up your schema (this should include both logs and prompts tables).
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(pool)
    }

    #[tokio::test]
    #[ignore]
    async fn test_openai_integration() {
        dotenv().ok();

        // Create a single shared pool.
        let pool = create_shared_in_memory_pool().await.unwrap();

        // Create your repositories using the shared pool.
        let prompt_repo = PromptRepository::in_memory(pool.clone()).await.unwrap();
        let log_repo = LogRepository::in_memory(pool.clone()).await.unwrap();

        // Seed your prompt.
        let prompt_id = prompt_repo
            .create_prompt(
                "test_key",
                "Test prompt content",
                "Test prompt content",
                1,
                100,
                0.5,
                false,
            )
            .await
            .unwrap();

        // Create the properties for your LLM request, using the prompt_id from the repository.
        let props = LlmProps {
            provider: LlmApiProvider::OpenAi,
            model_name: "gpt-4o-mini-2024-07-18".to_string(),
            temperature: 0.5,
            max_tokens: 100,
            json_mode: true,
            messages: vec![
                Message::System {
                    content: "You must respond with valid JSON only".to_string(),
                },
                Message::User {
                    content: "Return a JSON object with a 'message' field containing 'Hello in JSON'"
                        .to_string(),
                },
            ],
            prompt_id, // Use the seeded prompt_id here.
            model_id: 1,
        };

        // Create your LLM instance, passing in the repositories (or the shared pool as needed).
        let llm = Llm::new(props, log_repo);

        // Run your tests.
        let res = llm.text().await.unwrap();
        assert!(res.content.contains("Hello"));

        #[derive(serde::Deserialize)]
        struct TestResponse {
            message: String,
        }

        let response = llm.json().await.unwrap();
        let json: TestResponse = serde_json::from_str(&response.content).unwrap();
        assert_eq!(json.message, "Hello in JSON");
    }

    #[tokio::test]
    #[ignore]
    async fn test_anthropic_integration() {
        dotenv().ok();
        let props = LlmProps {
            provider: LlmApiProvider::Anthropic,
            model_name: "claude-3-5-haiku-latest".to_string(),
            temperature: 0.5,
            max_tokens: 100,
            json_mode: false,
            messages: vec![
                Message::System {
                    content: "You are a friendly assistance".to_string(),
                },
                Message::User {
                    content: "You return a message saying 'Hello'".to_string(),
                },
            ],
            prompt_id: 1,
            model_id: 1,
        };

        let pool = create_shared_in_memory_pool().await.unwrap();
        let prompt_repo = PromptRepository::in_memory(pool.clone()).await.unwrap();
        let log_repo = LogRepository::in_memory(pool.clone()).await.unwrap();

        // Seed your prompt.
        prompt_repo
            .create_prompt(
                "test_key",
                "Test prompt content",
                "Test prompt content",
                1,
                100,
                0.5,
                false,
            )
            .await
            .unwrap();

        let llm = Llm::new(props, log_repo);

        // Test text response - Anthropic doesn't support JSON mode
        let res = llm.text().await.unwrap();
        assert!(res.content.contains("Hello"));
    }

    #[tokio::test]
    #[ignore]
    async fn test_gemini_integration() {
        dotenv().ok();

        // Test text response
        let props = LlmProps {
            provider: LlmApiProvider::Gemini,
            model_name: "gemini-1.5-flash".to_string(),
            temperature: 0.5,
            max_tokens: 100,
            json_mode: false,
            messages: vec![
                Message::System {
                    content: "You are a friendly assistance".to_string(),
                },
                Message::User {
                    content: "You return a message saying 'Hello'".to_string(),
                },
            ],
            prompt_id: 1,
            model_id: 1,
        };

        let pool = create_shared_in_memory_pool().await.unwrap();
        let prompt_repo = PromptRepository::in_memory(pool.clone()).await.unwrap();
        let log_repo = LogRepository::in_memory(pool.clone()).await.unwrap();

        // Seed your prompt.
        prompt_repo
            .create_prompt(
                "test_key",
                "Test prompt content",
                "Test prompt content",
                1,
                100,
                0.5,
                false,
            )
            .await
            .unwrap();

        let llm = Llm::new(props, log_repo.clone());
        let res = llm.text().await.unwrap();
        assert!(res.content.contains("Hello"));

        // Test JSON response
        #[derive(serde::Deserialize)]
        struct TestResponse {
            message: String,
        }

        let props = LlmProps {
            provider: LlmApiProvider::Gemini,
            model_name: "gemini-1.5-flash".to_string(),
            temperature: 0.5,
            max_tokens: 100,
            json_mode: true,
            messages: vec![
                Message::System {
                    content: "You must respond with valid JSON only".to_string(),
                },
                Message::User {
                    content: "Return a JSON object with a 'message' field containing 'Hello in JSON'"
                        .to_string(),
                },
            ],
            prompt_id: 1,
            model_id: 1,
        };

        let llm = Llm::new(props, log_repo);
        let res = llm.json().await.unwrap();
        let json: TestResponse = serde_json::from_str(&res.content).unwrap();
        assert_eq!(json.message, "Hello in JSON");
    }

    #[tokio::test]
    #[ignore]
    async fn test_deepseek_integration() {
        dotenv().ok();

        // Test text response
        let props = create_test_props(
            LlmApiProvider::Deepseek,
            "deepseek-chat".to_string(),
        )
        .await;

        let pool = create_shared_in_memory_pool().await.unwrap();
        let prompt_repo = PromptRepository::in_memory(pool.clone()).await.unwrap();
        let log_repo = LogRepository::in_memory(pool.clone()).await.unwrap();

        // Seed your prompt.
        prompt_repo
            .create_prompt(
                "test_key",
                "Test prompt content",
                "Test prompt content",
                1,
                100,
                0.5,
                false,
            )
            .await
            .unwrap();

        let llm = Llm::new(props, log_repo.clone());
        let res = llm.text().await.unwrap();
        assert!(res.content.contains("Hello"));

        // Test JSON response
        #[derive(serde::Deserialize)]
        struct TestResponse {
            #[serde(rename = "content")]
            message: String,
        }

        let props = LlmProps {
            provider: LlmApiProvider::Deepseek,
            model_name: "deepseek-chat".to_string(),
            temperature: 0.5,
            max_tokens: 100,
            json_mode: true,
            messages: vec![
                Message::System {
                    content: "Respond with JSON containing a 'content' field".to_string(),
                },
                Message::User {
                    content: "Return JSON with format: {\"content\": \"Hello in JSON\"}".to_string(),
                },
            ],
            prompt_id: 1,
            model_id: 1,
        };

        let llm = Llm::new(props, log_repo);
        let res = llm.json().await.unwrap();
        let json: TestResponse = serde_json::from_str(&res.content).unwrap();
        assert_eq!(json.message, "Hello in JSON");
    }

    #[tokio::test]
    #[ignore]
    async fn test_azure_integration() {
        dotenv().ok();

        // Test text response
        let props = create_test_props(
            LlmApiProvider::Azure,
            "gpt-4o-mini".to_string(),
        )
        .await;

        let pool = create_shared_in_memory_pool().await.unwrap();
        let prompt_repo = PromptRepository::in_memory(pool.clone()).await.unwrap();
        let log_repo = LogRepository::in_memory(pool.clone()).await.unwrap();

        // Seed your prompt.
        prompt_repo
            .create_prompt(
                "test_key",
                "Test prompt content",
                "Test prompt content",
                1,
                100,
                0.5,
                false,
            )
            .await
            .unwrap();

        let llm = Llm::new(props, log_repo.clone());
        let res = llm.text().await.unwrap();
        assert!(res.content.contains("Hello"));

        // Test JSON response
        #[derive(serde::Deserialize)]
        struct TestResponse {
            #[serde(rename = "content")]
            message: String,
        }

        let props = LlmProps {
            provider: LlmApiProvider::Azure,
            model_name: "gpt-4o-mini".to_string(),
            temperature: 0.5,
            max_tokens: 100,
            json_mode: true,
            messages: vec![
                Message::System {
                    content: "Respond with JSON containing a 'content' field".to_string(),
                },
                Message::User {
                    content: "Return JSON with format: {\"content\": \"Hello in JSON\"}".to_string(),
                },
            ],
            prompt_id: 1,
            model_id: 1,
        };

        let llm = Llm::new(props, log_repo);
        let res = llm.json().await.unwrap();
        let json: TestResponse = serde_json::from_str(&res.content).unwrap();
        assert_eq!(json.message, "Hello in JSON");
    }

    async fn create_stream_test_props(provider: LlmApiProvider, model_name: String) -> LlmProps {
        LlmProps {
            provider,
            model_name,
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
        let props = create_stream_test_props(
            LlmApiProvider::OpenAi,
            "gpt-4o-mini-2024-07-18".to_string(),
        ).await;

        let pool = create_shared_in_memory_pool().await.unwrap();
        let prompt_repo = PromptRepository::in_memory(pool.clone()).await.unwrap();
        let log_repo = LogRepository::in_memory(pool.clone()).await.unwrap();

        // Seed your prompt.
        prompt_repo.create_prompt("", "", "", 1, 100, 0.5, false ).await.unwrap();

        let llm = Llm::new(props, log_repo.clone());
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
        let props = create_stream_test_props(
            LlmApiProvider::Anthropic,
            "claude-3-5-haiku-latest".to_string(),
        )
        .await;

        let pool = create_shared_in_memory_pool().await.unwrap();
        let prompt_repo = PromptRepository::in_memory(pool.clone()).await.unwrap();
        let log_repo = LogRepository::in_memory(pool.clone()).await.unwrap();

        // Seed your prompt.
        prompt_repo
            .create_prompt(
                "test_key",
                "Test prompt content",
                "Test prompt content",
                1,
                100,
                0.5,
                false,
            )
            .await
            .unwrap();

        let llm = Llm::new(props, log_repo.clone());
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
        let props = create_stream_test_props(
            LlmApiProvider::Gemini,
            "gemini-1.5-flash".to_string(),
        )
        .await;

        let pool = create_shared_in_memory_pool().await.unwrap();
        let prompt_repo = PromptRepository::in_memory(pool.clone()).await.unwrap();
        let log_repo = LogRepository::in_memory(pool.clone()).await.unwrap();

        // Seed your prompt.
        prompt_repo
            .create_prompt(
                "test_key",
                "Test prompt content",
                "Test prompt content",
                1,
                100,
                0.5,
                false,
            )
            .await
            .unwrap();

        let llm = Llm::new(props, log_repo.clone());

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
        let props = create_stream_test_props(
            LlmApiProvider::Deepseek,
            "deepseek-chat".to_string(),
        )
        .await;
        let pool = create_shared_in_memory_pool().await.unwrap();
        let prompt_repo = PromptRepository::in_memory(pool.clone()).await.unwrap();
        let log_repo = LogRepository::in_memory(pool.clone()).await.unwrap();

        // Seed your prompt.
        prompt_repo
            .create_prompt(
                "test_key",
                "Test prompt content",
                "Test prompt content",
                1,
                100,
                0.5,
                false,
            )
            .await
            .unwrap();

        let llm = Llm::new(props, log_repo.clone());
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
    async fn test_azure_stream() {
        dotenv().ok();
        let props = create_stream_test_props(
            LlmApiProvider::Azure,
            "gpt-4o-mini".to_string(),
        )
        .await;
        let pool = create_shared_in_memory_pool().await.unwrap();
        let prompt_repo = PromptRepository::in_memory(pool.clone()).await.unwrap();
        let log_repo = LogRepository::in_memory(pool.clone()).await.unwrap();

        // Seed your prompt.
        prompt_repo
            .create_prompt(
                "test_key",
                "Test prompt content",
                "Test prompt content",
                1,
                100,
                0.5,
                false,
            )
            .await
            .unwrap();

        let llm = Llm::new(props, log_repo.clone());
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

    async fn create_multi_turn_test_props(model: LlmApiProvider, model_name: String) -> LlmProps {
        LlmProps {
            provider: model,
            model_name,
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
                Message::Assistant {
                    content: "Hello world".to_string(),
                },
                Message::User {
                    content: "Great, now say 'What's up'".to_string(),
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
            (
                LlmApiProvider::OpenAi,
                "gpt-4o-mini-2024-07-18".to_string(),
            ),
            (
                LlmApiProvider::Anthropic,
                "claude-3-5-haiku-latest".to_string(),
            ),
            (
                LlmApiProvider::Gemini,
                "gemini-1.5-flash".to_string(),
            ),
            (
                LlmApiProvider::Deepseek,
                "deepseek-chat".to_string(),
            ),
        ];

        for (model, model_name) in models {
            let props = create_multi_turn_test_props(model.clone(), model_name).await;
            let pool = create_shared_in_memory_pool().await.unwrap();
            let prompt_repo = PromptRepository::in_memory(pool.clone()).await.unwrap();
            let log_repo = LogRepository::in_memory(pool.clone()).await.unwrap();

            // Seed your prompt.
            prompt_repo
                .create_prompt(
                    "test_key",
                    "Test prompt content",
                    "Test prompt content",
                    1,
                    100,
                    0.5,
                    false,
                )
                .await
                .unwrap();

            let llm = Llm::new(props, log_repo.clone());

            // The response should continue the conversation naturally
            let response = llm.text().await.unwrap();
            assert!(
                response.content.contains("What's"),
            );
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_multi_turn_stream() {
        dotenv().ok();

        let models = vec![
            (
                LlmApiProvider::OpenAi,
                "gpt-4o-mini-2024-07-18".to_string(),
            ),
            (
                LlmApiProvider::Anthropic,
                "claude-3-5-haiku-latest".to_string(),
            ),
            (
                LlmApiProvider::Gemini,
                "gemini-1.5-flash".to_string(),
            ),
            (
                LlmApiProvider::Deepseek,
                "deepseek-chat".to_string(),
            ),
            (
                LlmApiProvider::Azure,
                "gpt-4o-mini".to_string(),
            ),
        ];

        for (model, model_name) in models {
            let props = create_multi_turn_test_props(model.clone(), model_name).await;
            let pool = create_shared_in_memory_pool().await.unwrap();
            let prompt_repo = PromptRepository::in_memory(pool.clone()).await.unwrap();
            let log_repo = LogRepository::in_memory(pool.clone()).await.unwrap();

            // Seed your prompt.
            prompt_repo
                .create_prompt(
                    "test_key",
                    "Test prompt content",
                    "Test prompt content",
                    1,
                    100,
                    0.5,
                    false,
                )
                .await
                .unwrap();

            let llm = Llm::new(props, log_repo.clone());
            let (tx, mut rx) = mpsc::channel(10);

            llm.stream(tx).await.expect("Streaming failed");

            let mut received_chunks = Vec::new();
            while let Some(result) = rx.recv().await {
                let chunk = result.expect("Failed to receive chunk");
                received_chunks.push(chunk);
            }

            assert!(
                !received_chunks.is_empty(),
                "Should receive chunks for model {:?}",
                model
            );
            let combined = received_chunks.join("");
            assert!(
                combined.contains("What's"),
            );
        }
    }
}

