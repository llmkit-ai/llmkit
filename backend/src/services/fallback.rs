use std::time::Duration;

use anyhow::Result;
use tokio::sync::mpsc::Sender;
use tokio_retry::{
    strategy::{jitter, ExponentialBackoff},
    Retry,
};
use tracing;

use super::{
    providers::{openai::OpenAiProvider, openrouter::OpenrouterProvider},
    types::{
        llm_error::{LlmError, LlmStreamingError}, 
        llm_service::{LlmServiceRequest, FallbackConfig, FallbackProviderConfig, FallbackErrorType}
    },
};
use crate::{
    common::types::{
        chat_response::{LlmServiceChatCompletionChunk, LlmServiceChatCompletionResponse}, 
        models::LlmApiProvider
    },
    db::logs::LogRepository
};

/// Information about a provider attempt during fallback
#[derive(Debug, Clone)]
pub struct ProviderAttempt {
    pub provider: LlmApiProvider,
    pub model_name: String,
    pub error: Option<String>,
    pub success: bool,
}

/// Result of fallback execution with metadata
#[derive(Debug)]
pub struct FallbackResult {
    pub response: LlmServiceChatCompletionResponse,
    pub log_id: i64,
    pub provider_attempts: Vec<ProviderAttempt>,
    pub caught_error: Option<LlmError>,
}

/// Service for handling fallback logic across multiple providers
pub struct FallbackService {
    original_request: LlmServiceRequest,
    db_log: LogRepository,
}

impl FallbackService {
    pub fn new(request: LlmServiceRequest, db_log: LogRepository) -> Self {
        FallbackService {
            original_request: request,
            db_log,
        }
    }

    /// Execute streaming request with fallback support
    pub async fn execute_stream_with_fallback(
        &self,
        tx: Sender<Result<LlmServiceChatCompletionChunk, LlmStreamingError>>,
    ) -> Result<FallbackResult, LlmError> {
        let fallback_config = match &self.original_request.fallback_config {
            Some(config) if config.enabled => config,
            _ => {
                // No fallback configured, execute normally
                return self.execute_single_provider_stream(&self.original_request, tx).await
                    .map(|(response, log_id)| FallbackResult {
                        response,
                        log_id,
                        provider_attempts: vec![ProviderAttempt {
                            provider: self.original_request.provider.clone(),
                            model_name: self.original_request.request.model.clone(),
                            error: None,
                            success: true,
                        }],
                        caught_error: None,
                    });
            }
        };

        let mut provider_attempts = Vec::new();
        let mut last_error = None;

        // Try the primary provider first
        match self.try_provider_stream_with_retries(&self.original_request, tx.clone(), fallback_config.max_retries_per_provider).await {
            Ok((response, log_id)) => {
                provider_attempts.push(ProviderAttempt {
                    provider: self.original_request.provider.clone(),
                    model_name: self.original_request.request.model.clone(),
                    error: None,
                    success: true,
                });

                return Ok(FallbackResult {
                    response,
                    log_id,
                    provider_attempts,
                    caught_error: None,
                });
            }
            Err(error) => {
                provider_attempts.push(ProviderAttempt {
                    provider: self.original_request.provider.clone(),
                    model_name: self.original_request.request.model.clone(),
                    error: Some(error.to_string()),
                    success: false,
                });

                tracing::warn!("Primary provider failed: {}, attempting fallbacks", error);
                last_error = Some(error.clone());

                // Check if this error should trigger fallback
                if !self.should_fallback(&error, fallback_config) {
                    tracing::info!("Error type does not trigger fallback, failing immediately");
                    return Err(error);
                }
            }
        }

        // Try each fallback provider
        for fallback_provider in &fallback_config.providers {
            if !self.should_try_fallback_provider(&last_error.as_ref().unwrap(), fallback_provider) {
                continue;
            }

            tracing::info!("Attempting fallback to provider: {:?} with model: {}", 
                fallback_provider.provider, fallback_provider.model_name);

            let fallback_request = self.create_fallback_request(fallback_provider);

            match self.try_provider_stream_with_retries(&fallback_request, tx.clone(), fallback_config.max_retries_per_provider).await {
                Ok((response, log_id)) => {
                    provider_attempts.push(ProviderAttempt {
                        provider: fallback_provider.provider.clone(),
                        model_name: fallback_provider.model_name.clone(),
                        error: None,
                        success: true,
                    });

                    return Ok(FallbackResult {
                        response,
                        log_id,
                        provider_attempts,
                        caught_error: last_error,
                    });
                }
                Err(error) => {
                    provider_attempts.push(ProviderAttempt {
                        provider: fallback_provider.provider.clone(),
                        model_name: fallback_provider.model_name.clone(),
                        error: Some(error.to_string()),
                        success: false,
                    });

                    tracing::warn!("Fallback provider {:?} failed: {}", fallback_provider.provider, error);
                    last_error = Some(error);
                }
            }
        }

        // All providers failed
        let attempted_providers = provider_attempts.iter()
            .map(|attempt| format!("{}({})", String::from(attempt.provider.clone()), attempt.model_name))
            .collect::<Vec<_>>()
            .join(", ");

        let provider_errors = provider_attempts.iter()
            .filter_map(|attempt| {
                attempt.error.as_ref().map(|error| {
                    (format!("{}({})", String::from(attempt.provider.clone()), attempt.model_name), error.clone())
                })
            })
            .collect();

        Err(LlmError::FallbackExhausted {
            attempted_providers,
            last_error: Box::new(last_error.unwrap()),
            provider_errors,
        })
    }

    /// Execute request with fallback support
    pub async fn execute_with_fallback(&self) -> Result<FallbackResult, LlmError> {
        let fallback_config = match &self.original_request.fallback_config {
            Some(config) if config.enabled => config,
            _ => {
                // No fallback configured, execute normally
                return self.execute_single_provider(&self.original_request).await
                    .map(|(response, log_id)| FallbackResult {
                        response,
                        log_id,
                        provider_attempts: vec![ProviderAttempt {
                            provider: self.original_request.provider.clone(),
                            model_name: self.original_request.request.model.clone(),
                            error: None,
                            success: true,
                        }],
                        caught_error: None,
                    });
            }
        };

        let mut provider_attempts = Vec::new();
        let mut last_error = None;

        // Try the primary provider first
        match self.try_provider_with_retries(&self.original_request, fallback_config.max_retries_per_provider).await {
            Ok((response, log_id)) => {
                provider_attempts.push(ProviderAttempt {
                    provider: self.original_request.provider.clone(),
                    model_name: self.original_request.request.model.clone(),
                    error: None,
                    success: true,
                });

                return Ok(FallbackResult {
                    response,
                    log_id,
                    provider_attempts,
                    caught_error: None,
                });
            }
            Err(error) => {
                provider_attempts.push(ProviderAttempt {
                    provider: self.original_request.provider.clone(),
                    model_name: self.original_request.request.model.clone(),
                    error: Some(error.to_string()),
                    success: false,
                });

                tracing::warn!("Primary provider failed: {}, attempting fallbacks", error);
                last_error = Some(error.clone());

                // Check if this error should trigger fallback
                if !self.should_fallback(&error, fallback_config) {
                    tracing::info!("Error type does not trigger fallback, failing immediately");
                    return Err(error);
                }
            }
        }

        // Try each fallback provider
        for fallback_provider in &fallback_config.providers {
            if !self.should_try_fallback_provider(&last_error.as_ref().unwrap(), fallback_provider) {
                continue;
            }

            tracing::info!("Attempting fallback to provider: {:?} with model: {}", 
                fallback_provider.provider, fallback_provider.model_name);

            let fallback_request = self.create_fallback_request(fallback_provider);

            match self.try_provider_with_retries(&fallback_request, fallback_config.max_retries_per_provider).await {
                Ok((response, log_id)) => {
                    provider_attempts.push(ProviderAttempt {
                        provider: fallback_provider.provider.clone(),
                        model_name: fallback_provider.model_name.clone(),
                        error: None,
                        success: true,
                    });

                    return Ok(FallbackResult {
                        response,
                        log_id,
                        provider_attempts,
                        caught_error: last_error,
                    });
                }
                Err(error) => {
                    provider_attempts.push(ProviderAttempt {
                        provider: fallback_provider.provider.clone(),
                        model_name: fallback_provider.model_name.clone(),
                        error: Some(error.to_string()),
                        success: false,
                    });

                    tracing::warn!("Fallback provider {:?} failed: {}", fallback_provider.provider, error);
                    last_error = Some(error);
                }
            }
        }

        // All providers failed
        let attempted_providers = provider_attempts.iter()
            .map(|attempt| format!("{}({})", String::from(attempt.provider.clone()), attempt.model_name))
            .collect::<Vec<_>>()
            .join(", ");

        let provider_errors = provider_attempts.iter()
            .filter_map(|attempt| {
                attempt.error.as_ref().map(|error| {
                    (format!("{}({})", String::from(attempt.provider.clone()), attempt.model_name), error.clone())
                })
            })
            .collect();

        Err(LlmError::FallbackExhausted {
            attempted_providers,
            last_error: Box::new(last_error.unwrap()),
            provider_errors,
        })
    }

    /// Check if the given error should trigger fallback based on configuration
    fn should_fallback(&self, error: &LlmError, config: &FallbackConfig) -> bool {
        config.providers.iter().any(|provider| {
            provider.catch_errors.iter().any(|error_type| {
                error_type.matches_error(error)
            })
        })
    }

    /// Check if we should try a specific fallback provider for the given error
    fn should_try_fallback_provider(&self, error: &LlmError, provider: &FallbackProviderConfig) -> bool {
        provider.catch_errors.iter().any(|error_type| {
            error_type.matches_error(error)
        })
    }

    /// Create a fallback request with the fallback provider configuration
    fn create_fallback_request(&self, fallback_provider: &FallbackProviderConfig) -> LlmServiceRequest {
        let mut fallback_request = self.original_request.clone();
        
        // Update provider-specific settings
        fallback_request.provider = fallback_provider.provider.clone();
        fallback_request.base_url = fallback_provider.base_url.clone();
        fallback_request.request.model = fallback_provider.model_name.clone();
        
        if let Some(max_tokens) = fallback_provider.max_tokens {
            fallback_request.request.max_tokens = Some(max_tokens);
        }
        
        if let Some(temperature) = fallback_provider.temperature {
            fallback_request.request.temperature = Some(temperature);
        }

        // Disable fallback for the fallback request to prevent infinite recursion
        fallback_request.fallback_config = None;

        fallback_request
    }

    /// Try a provider with retry logic
    async fn try_provider_with_retries(
        &self, 
        request: &LlmServiceRequest, 
        max_retries: usize
    ) -> Result<(LlmServiceChatCompletionResponse, i64), LlmError> {
        let retry_strategy = ExponentialBackoff::from_millis(100)
            .max_delay(Duration::from_secs(3))
            .map(jitter)
            .take(max_retries);

        Retry::spawn(retry_strategy, || self.execute_single_provider(request)).await
    }

    /// Try a provider with retry logic for streaming
    async fn try_provider_stream_with_retries(
        &self, 
        request: &LlmServiceRequest,
        tx: Sender<Result<LlmServiceChatCompletionChunk, LlmStreamingError>>,
        max_retries: usize
    ) -> Result<(LlmServiceChatCompletionResponse, i64), LlmError> {
        let retry_strategy = ExponentialBackoff::from_millis(100)
            .max_delay(Duration::from_secs(3))
            .map(jitter)
            .take(max_retries);

        Retry::spawn(retry_strategy, || self.execute_single_provider_stream(request, tx.clone())).await
    }

    /// Execute request for a single provider without fallback
    async fn execute_single_provider(
        &self, 
        request: &LlmServiceRequest
    ) -> Result<(LlmServiceChatCompletionResponse, i64), LlmError> {
        // Initialize variables to capture data even in error cases
        let mut input_tokens = None;
        let mut output_tokens = None;
        let mut reasoning_tokens = None;
        let mut raw_response: Option<String> = None;
        let mut status = Some(500); // Default to error status

        // Serialize the request for logging
        let request_body = serde_json::to_string(&request)
            .map_err(|e| LlmError::SerializationError(e.to_string()))?;

        // Execute request and capture result
        let result = match &request.provider {
            LlmApiProvider::Openrouter => {
                let provider = OpenrouterProvider::new(&request)?;
                provider.execute_chat().await
            }
            LlmApiProvider::OpenAi => {
                let provider = OpenAiProvider::new(&request)?;
                provider.execute_chat().await
            }
            LlmApiProvider::Azure => {
                let provider = OpenAiProvider::new_azure(&request)?;
                provider.execute_chat().await
            }
        };

        // Process the result or prepare error
        let (exec_result, provider_response_id) = match result {
            Ok(provider_response) => {
                // Update status for successful request
                status = Some(200);

                // Extract tokens and usage information
                input_tokens = provider_response
                    .usage
                    .as_ref()
                    .map(|usage| usage.prompt_tokens as i64);

                output_tokens = provider_response
                    .usage
                    .as_ref()
                    .map(|usage| usage.completion_tokens as i64);

                // Extract reasoning tokens if available
                reasoning_tokens = provider_response
                    .usage
                    .as_ref()
                    .and_then(|usage| usage.completion_tokens_details.as_ref())
                    .and_then(|details| details.reasoning_tokens)
                    .map(|tokens| tokens as i64);

                // Save raw response for logging
                raw_response = serde_json::to_string(&provider_response).ok();

                // Extract content from the response
                if provider_response.choices.len() > 0 {
                    // Save response ID for logging
                    let id = provider_response.id.clone();
                    (Ok(provider_response), id)
                } else {
                    (Err(LlmError::EmptyResponse), uuid::Uuid::new_v4().to_string())
                }
            }
            Err(e) => {
                // For errors, prepare as much information as possible for logging
                raw_response = Some(format!("{{\"error\": \"{}\"}}", e));
                (Err(e), uuid::Uuid::new_v4().to_string())
            }
        };

        // Always log the request, regardless of success or failure
        let log_id = self
            .log_request(
                raw_response.as_deref(),
                status,
                input_tokens,
                output_tokens,
                reasoning_tokens,
                &request_body,
                &provider_response_id,
                &request.prompt_id,
                &request.model_id,
            )
            .await?;

        // Return the original result but with the correct log_id
        match exec_result {
            Ok(r) => Ok((r, log_id)),
            Err(e) => Err(e),
        }
    }

    /// Execute streaming request for a single provider without fallback
    async fn execute_single_provider_stream(
        &self, 
        request: &LlmServiceRequest,
        tx: Sender<Result<LlmServiceChatCompletionChunk, LlmStreamingError>>,
    ) -> Result<(LlmServiceChatCompletionResponse, i64), LlmError> {
        // Initialize variables to capture data even in error cases
        let mut input_tokens = None;
        let mut output_tokens = None;
        let mut reasoning_tokens = None;
        let mut raw_response: Option<String> = None;
        let mut status = Some(500); // Default to error status

        // Serialize the request for logging
        let request_body = serde_json::to_string(&request)
            .map_err(|e| LlmError::SerializationError(e.to_string()))?;

        // Check json mode before making the request
        if request.request.response_format.is_some() {
            tracing::info!("Json mode not supported in chat mode");
            let error = LlmError::UnsupportedMode("Json".to_string(), "Chat".to_string());

            // Log the error
            raw_response = Some(format!("{{\"error\": \"{}\"}}", error));
            let provider_response_id = uuid::Uuid::new_v4().to_string();

            // Log the failed request
            self.log_request(
                raw_response.as_deref(),
                status,
                input_tokens,
                output_tokens,
                reasoning_tokens,
                &request_body,
                &provider_response_id,
                &request.prompt_id,
                &request.model_id,
            )
            .await?;

            return Err(error);
        }

        // Execute request and capture result
        let result = match &request.provider {
            LlmApiProvider::Openrouter => {
                let provider = OpenrouterProvider::new(&request)?;
                provider.execute_chat_stream(tx).await
            }
            LlmApiProvider::OpenAi => {
                let provider = OpenAiProvider::new(&request)?;
                provider.execute_chat_stream(tx).await
            }
            LlmApiProvider::Azure => {
                let provider = OpenAiProvider::new_azure(&request)?;
                provider.execute_chat_stream(tx).await
            }
        };

        // Process the result or prepare error
        let (exec_result, provider_response_id) = match result {
            Ok(response) => {
                // Update status for successful request
                status = Some(200);

                // Extract tokens and usage information
                input_tokens = response
                    .usage
                    .as_ref()
                    .map(|usage| usage.prompt_tokens as i64);

                output_tokens = response
                    .usage
                    .as_ref()
                    .map(|usage| usage.completion_tokens as i64);

                // Extract reasoning tokens if available
                reasoning_tokens = response
                    .usage
                    .as_ref()
                    .and_then(|usage| usage.completion_tokens_details.as_ref())
                    .and_then(|details| details.reasoning_tokens)
                    .map(|tokens| tokens as i64);

                // Save raw response for logging
                raw_response = serde_json::to_string(&response).ok();

                // Extract content from the response
                if response.choices.len() > 0 {
                    let id = response.id.clone();
                    (Ok(response), id)
                } else {
                    (Err(LlmError::EmptyResponse), uuid::Uuid::new_v4().to_string())
                }
            }
            Err(e) => {
                // For errors, prepare as much information as possible for logging
                raw_response = Some(format!("{{\"error\": \"{}\"}}", e));
                (Err(e), uuid::Uuid::new_v4().to_string())
            }
        };

        // Always log the request, regardless of success or failure
        let log_id = self
            .log_request(
                raw_response.as_deref(),
                status,
                input_tokens,
                output_tokens,
                reasoning_tokens,
                &request_body,
                &provider_response_id,
                &request.prompt_id,
                &request.model_id,
            )
            .await?;

        // Return the original result but with the correct log_id
        match exec_result {
            Ok(r) => Ok((r, log_id)),
            Err(e) => Err(e),
        }
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
        provider_response_id: &str,
        prompt_id: &i64,
        model_id: &i64,
    ) -> Result<i64, LlmError> {
        self.db_log
            .create_log(
                Some(*prompt_id),
                *model_id,
                raw_response,
                status,
                input_tokens,
                output_tokens,
                reasoning_tokens,
                Some(request_body),
                provider_response_id,
            )
            .await
            .map_err(|e| LlmError::DbLoggingError(e.to_string()))
    }
}

// Tests will be added later after integration is complete