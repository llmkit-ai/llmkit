// Example demonstrating how to use the fallback functionality
// This example shows how to configure fallback providers and error handling

use backend::services::{
    llm::Llm,
    types::llm_service::{
        LlmServiceRequest, FallbackConfig, FallbackProviderConfig, FallbackErrorType
    }
};
use backend::common::types::{
    chat_request::{ChatCompletionRequest, ChatCompletionRequestMessage},
    models::LlmApiProvider
};
use backend::db::logs::LogRepository;

// Example 1: Basic fallback configuration
pub fn create_basic_fallback_config() -> FallbackConfig {
    FallbackConfig {
        enabled: true,
        providers: vec![
            // First fallback: OpenRouter on rate limit or auth errors
            FallbackProviderConfig {
                provider: LlmApiProvider::Openrouter,
                model_name: "openai/gpt-3.5-turbo".to_string(),
                base_url: Some("https://openrouter.ai/api/v1".to_string()),
                max_tokens: Some(1000),
                temperature: Some(0.7),
                catch_errors: vec![
                    FallbackErrorType::RateLimit,
                    FallbackErrorType::Auth,
                ],
            },
            // Second fallback: Azure OpenAI on any error
            FallbackProviderConfig {
                provider: LlmApiProvider::Azure,
                model_name: "gpt-4|2024-02-15-preview".to_string(), // Azure format: model|version
                base_url: Some("https://your-resource.openai.azure.com".to_string()),
                max_tokens: Some(1000),
                temperature: Some(0.7),
                catch_errors: vec![FallbackErrorType::All],
            },
        ],
        max_retries_per_provider: 3,
    }
}

// Example 2: Specific error type fallback
pub fn create_specific_error_fallback_config() -> FallbackConfig {
    FallbackConfig {
        enabled: true,
        providers: vec![
            // Only fallback on rate limits to OpenRouter
            FallbackProviderConfig {
                provider: LlmApiProvider::Openrouter,
                model_name: "anthropic/claude-3-sonnet".to_string(),
                base_url: None, // Will use default OpenRouter URL
                max_tokens: None, // Will use original request settings
                temperature: None, // Will use original request settings
                catch_errors: vec![FallbackErrorType::RateLimit],
            },
            // Only fallback on auth errors to Azure
            FallbackProviderConfig {
                provider: LlmApiProvider::Azure,
                model_name: "gpt-35-turbo|2024-02-15-preview".to_string(),
                base_url: Some("https://your-resource.openai.azure.com".to_string()),
                max_tokens: Some(2000),
                temperature: Some(0.5),
                catch_errors: vec![
                    FallbackErrorType::Auth,
                    FallbackErrorType::ProviderUnavailable,
                ],
            },
        ],
        max_retries_per_provider: 2,
    }
}

// Example 3: How to use fallback in practice
pub async fn example_usage() -> Result<(), Box<dyn std::error::Error>> {
    // Create a sample request
    let mut request = LlmServiceRequest {
        provider: LlmApiProvider::OpenAi, // Primary provider
        base_url: None,
        prompt_id: 1,
        model_id: 1,
        is_reasoning: false,
        reasoning_effort: None,
        request: ChatCompletionRequest {
            model: "gpt-4".to_string(),
            messages: vec![
                ChatCompletionRequestMessage::System {
                    content: "You are a helpful assistant.".to_string(),
                    name: None,
                },
                ChatCompletionRequestMessage::User {
                    content: "What is the capital of France?".to_string(),
                    name: None,
                },
            ],
            stream: None,
            response_format: None,
            tools: None,
            provider: None,
            models: None,
            transforms: None,
            max_tokens: Some(100),
            temperature: Some(0.7),
        },
        fallback_config: Some(create_basic_fallback_config()), // Add fallback configuration
    };

    // Create database connection and log repository
    // Note: In real usage, you'd use your actual database connection
    let db_pool = std::sync::Arc::new(
        sqlx::SqlitePool::connect(":memory:")
            .await
            .expect("Failed to create database connection")
    );
    let log_repo = LogRepository::new(db_pool);

    // Create LLM service
    let llm_service = Llm::new(request, log_repo);

    // Execute the request with fallback support
    match llm_service.text().await {
        Ok((response, log_id)) => {
            println!("✅ Request successful!");
            println!("Response ID: {}", response.id);
            println!("Log ID: {}", log_id);
            
            if let Some(choice) = response.choices.first() {
                if let Some(content) = &choice.message.content {
                    println!("Response: {}", content);
                }
            }
        }
        Err(backend::services::types::llm_error::LlmError::FallbackExhausted { 
            attempted_providers, 
            last_error, 
            provider_errors 
        }) => {
            println!("❌ All fallback providers failed!");
            println!("Attempted providers: {}", attempted_providers);
            println!("Last error: {}", last_error);
            println!("Provider errors: {:?}", provider_errors);
        }
        Err(e) => {
            println!("❌ Request failed: {}", e);
        }
    }

    Ok(())
}

// Example 4: Streaming with fallback
pub async fn example_streaming_usage() -> Result<(), Box<dyn std::error::Error>> {
    use tokio::sync::mpsc;
    use backend::services::types::llm_error::LlmStreamingError;
    use backend::common::types::chat_response::LlmServiceChatCompletionChunk;

    // Create request with fallback
    let mut request = LlmServiceRequest {
        provider: LlmApiProvider::OpenAi,
        base_url: None,
        prompt_id: 1,
        model_id: 1,
        is_reasoning: false,
        reasoning_effort: None,
        request: ChatCompletionRequest {
            model: "gpt-4".to_string(),
            messages: vec![
                ChatCompletionRequestMessage::User {
                    content: "Tell me a short story".to_string(),
                    name: None,
                },
            ],
            stream: Some(true),
            response_format: None,
            tools: None,
            provider: None,
            models: None,
            transforms: None,
            max_tokens: Some(500),
            temperature: Some(0.8),
        },
        fallback_config: Some(create_basic_fallback_config()),
    };

    // Create database connection and log repository
    let db_pool = std::sync::Arc::new(
        sqlx::SqlitePool::connect(":memory:")
            .await
            .expect("Failed to create database connection")
    );
    let log_repo = LogRepository::new(db_pool);

    // Create channel for streaming
    let (tx, mut rx) = mpsc::channel::<Result<LlmServiceChatCompletionChunk, LlmStreamingError>>(100);

    // Create LLM service
    let llm_service = Llm::new(request, log_repo);

    // Start streaming with fallback support
    let stream_handle = tokio::spawn(async move {
        llm_service.stream(tx).await
    });

    // Process streaming chunks
    while let Some(chunk_result) = rx.recv().await {
        match chunk_result {
            Ok(chunk) => {
                if let Some(choice) = chunk.choices.first() {
                    if let Some(content) = &choice.delta.content {
                        print!("{}", content); // Print streaming content
                    }
                }
            }
            Err(e) => {
                println!("Streaming error: {:?}", e);
                break;
            }
        }
    }

    // Wait for the stream to complete
    match stream_handle.await? {
        Ok((response, log_id)) => {
            println!("\n✅ Streaming completed successfully!");
            println!("Response ID: {}", response.id);
            println!("Log ID: {}", log_id);
        }
        Err(e) => {
            println!("❌ Streaming failed: {}", e);
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    println!("🚀 Running fallback examples...\n");

    println!("📝 Example 1: Basic text generation with fallback");
    example_usage().await?;

    println!("\n📝 Example 2: Streaming with fallback");
    example_streaming_usage().await?;

    println!("\n✨ Examples completed!");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_config_creation() {
        let config = create_basic_fallback_config();
        assert!(config.enabled);
        assert_eq!(config.providers.len(), 2);
        assert_eq!(config.max_retries_per_provider, 3);
    }

    #[test]
    fn test_specific_error_fallback_config() {
        let config = create_specific_error_fallback_config();
        assert!(config.enabled);
        assert_eq!(config.providers.len(), 2);
        
        // Check first provider only catches rate limits
        assert_eq!(config.providers[0].catch_errors.len(), 1);
        assert_eq!(config.providers[0].catch_errors[0], FallbackErrorType::RateLimit);
        
        // Check second provider catches auth and unavailable errors
        assert_eq!(config.providers[1].catch_errors.len(), 2);
        assert!(config.providers[1].catch_errors.contains(&FallbackErrorType::Auth));
        assert!(config.providers[1].catch_errors.contains(&FallbackErrorType::ProviderUnavailable));
    }
}