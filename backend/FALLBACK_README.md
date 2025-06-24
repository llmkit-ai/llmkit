# LLM Fallback Support

This document describes the fallback functionality that allows automatic switching between different LLM providers when errors occur, similar to the functionality provided by Mirascope in Python.

## Overview

The fallback system automatically attempts to use backup providers when the primary provider fails due to specific error conditions like rate limits, authentication errors, or provider outages. This increases the reliability and availability of your LLM applications.

## Features

- **Multiple Provider Support**: Fallback between OpenAI, Azure OpenAI, and OpenRouter
- **Configurable Error Triggers**: Specify which errors should trigger fallbacks
- **Retry Logic**: Each provider can be retried multiple times before falling back
- **Streaming Support**: Works with both regular and streaming requests
- **Detailed Logging**: Tracks all attempted providers and their errors
- **Error Context**: Preserves information about caught errors for debugging

## Quick Start

### 1. Basic Configuration

```rust
use backend::services::types::llm_service::{
    FallbackConfig, FallbackProviderConfig, FallbackErrorType
};
use backend::common::types::models::LlmApiProvider;

let fallback_config = FallbackConfig {
    enabled: true,
    providers: vec![
        FallbackProviderConfig {
            provider: LlmApiProvider::Openrouter,
            model_name: "openai/gpt-3.5-turbo".to_string(),
            base_url: None,
            max_tokens: None,
            temperature: None,
            catch_errors: vec![FallbackErrorType::RateLimit, FallbackErrorType::Auth],
        },
    ],
    max_retries_per_provider: 3,
};
```

### 2. Adding Fallback to a Request

```rust
let mut request = LlmServiceRequest {
    provider: LlmApiProvider::OpenAi, // Primary provider
    // ... other fields
    fallback_config: Some(fallback_config),
};

let llm = Llm::new(request, log_repository);
let result = llm.text().await?;
```

## Configuration Options

### FallbackConfig

- `enabled: bool` - Whether fallback is enabled
- `providers: Vec<FallbackProviderConfig>` - List of fallback providers
- `max_retries_per_provider: usize` - Number of retries per provider before moving to next

### FallbackProviderConfig

- `provider: LlmApiProvider` - The provider to use (OpenAI, Azure, OpenRouter)
- `model_name: String` - Model name for this provider
- `base_url: Option<String>` - Custom base URL (optional)
- `max_tokens: Option<u32>` - Override max tokens (optional)
- `temperature: Option<f32>` - Override temperature (optional)
- `catch_errors: Vec<FallbackErrorType>` - Which errors trigger this fallback

### Error Types

Available error types that can trigger fallbacks:

- `FallbackErrorType::RateLimit` - Rate limit exceeded
- `FallbackErrorType::Auth` - Authentication errors
- `FallbackErrorType::ProviderUnavailable` - Provider is unavailable
- `FallbackErrorType::ProviderQuotaExceeded` - Provider quota exceeded
- `FallbackErrorType::Timeout` - Request timeout
- `FallbackErrorType::Network` - Network errors
- `FallbackErrorType::All` - Any error (catch-all)

## Usage Examples

### Example 1: Rate Limit Fallback

```rust
// Fallback to OpenRouter on OpenAI rate limits
FallbackConfig {
    enabled: true,
    providers: vec![
        FallbackProviderConfig {
            provider: LlmApiProvider::Openrouter,
            model_name: "openai/gpt-4".to_string(),
            base_url: None,
            max_tokens: None,
            temperature: None,
            catch_errors: vec![FallbackErrorType::RateLimit],
        },
    ],
    max_retries_per_provider: 2,
}
```

### Example 2: Multi-Provider Chain

```rust
// Chain of fallbacks: OpenAI -> OpenRouter -> Azure
FallbackConfig {
    enabled: true,
    providers: vec![
        // First fallback: OpenRouter for rate limits
        FallbackProviderConfig {
            provider: LlmApiProvider::Openrouter,
            model_name: "openai/gpt-3.5-turbo".to_string(),
            catch_errors: vec![FallbackErrorType::RateLimit],
            // ... other fields
        },
        // Second fallback: Azure for any remaining errors
        FallbackProviderConfig {
            provider: LlmApiProvider::Azure,
            model_name: "gpt-35-turbo|2024-02-15-preview".to_string(),
            base_url: Some("https://your-resource.openai.azure.com".to_string()),
            catch_errors: vec![FallbackErrorType::All],
            // ... other fields
        },
    ],
    max_retries_per_provider: 3,
}
```

### Example 3: Streaming with Fallback

```rust
use tokio::sync::mpsc;

let (tx, mut rx) = mpsc::channel(100);
let llm = Llm::new(request_with_fallback, log_repository);

// Start streaming
let handle = tokio::spawn(async move {
    llm.stream(tx).await
});

// Process chunks
while let Some(chunk) = rx.recv().await {
    match chunk {
        Ok(chunk) => {
            // Process streaming chunk
            if let Some(content) = chunk.choices.first()
                .and_then(|c| c.delta.content.as_ref()) {
                print!("{}", content);
            }
        }
        Err(e) => {
            eprintln!("Streaming error: {}", e);
            break;
        }
    }
}

// Wait for completion
let result = handle.await??;
```

## Error Handling

When all fallback providers fail, a `FallbackExhausted` error is returned:

```rust
match llm.text().await {
    Ok((response, log_id)) => {
        // Success
        println!("Response: {:?}", response);
    }
    Err(LlmError::FallbackExhausted { 
        attempted_providers, 
        last_error, 
        provider_errors 
    }) => {
        println!("All providers failed!");
        println!("Attempted: {}", attempted_providers);
        println!("Last error: {}", last_error);
        println!("All errors: {:?}", provider_errors);
    }
    Err(e) => {
        // Other error (fallback not triggered)
        println!("Error: {}", e);
    }
}
```

## Provider-Specific Configuration

### OpenAI
```rust
FallbackProviderConfig {
    provider: LlmApiProvider::OpenAi,
    model_name: "gpt-4".to_string(),
    base_url: None, // Uses default OpenAI API
    // Requires OPENAI_API_KEY environment variable
}
```

### Azure OpenAI
```rust
FallbackProviderConfig {
    provider: LlmApiProvider::Azure,
    model_name: "gpt-4|2024-02-15-preview".to_string(), // model|version format
    base_url: Some("https://your-resource.openai.azure.com".to_string()),
    // Requires AZURE_API_KEY environment variable
}
```

### OpenRouter
```rust
FallbackProviderConfig {
    provider: LlmApiProvider::Openrouter,
    model_name: "openai/gpt-4".to_string(), // OpenRouter model format
    base_url: Some("https://openrouter.ai/api/v1".to_string()),
    // Uses OpenRouter API key configuration
}
```

## Best Practices

1. **Order Providers by Cost**: Place cheaper providers first in the fallback chain
2. **Use Specific Error Types**: Only trigger fallback for recoverable errors
3. **Set Reasonable Retry Limits**: Balance between reliability and response time
4. **Monitor Provider Usage**: Track which providers are being used most
5. **Configure Rate Limits**: Ensure your API keys have appropriate rate limits
6. **Test Fallback Scenarios**: Regularly test that fallback works as expected

## Logging and Monitoring

The fallback system provides detailed logging:

```rust
// Enable tracing to see fallback logs
tracing_subscriber::fmt::init();

// Logs will show:
// - Primary provider attempts
// - Fallback trigger conditions
// - Provider switching
// - Final success/failure
```

## Environment Variables

Ensure you have the required API keys set:

```bash
# For OpenAI
export OPENAI_API_KEY="your-openai-key"

# For Azure OpenAI
export AZURE_API_KEY="your-azure-key"

# For OpenRouter (if using openrouter_api crate)
export OPENROUTER_API_KEY="your-openrouter-key"
```

## Limitations

- **No Automatic Model Matching**: Different providers may have different model names
- **Streaming Limitations**: Some providers may not support identical streaming features
- **Rate Limit Coordination**: Fallback doesn't coordinate rate limits across providers
- **Response Format Differences**: Slight differences in response format between providers

## Troubleshooting

### Common Issues

1. **Fallback Not Triggered**
   - Check if the error type matches your `catch_errors` configuration
   - Verify `enabled: true` in your FallbackConfig

2. **All Providers Failing**
   - Check API keys are correctly set
   - Verify network connectivity
   - Check provider-specific base URLs

3. **Unexpected Model Behavior**
   - Different providers may have different model behaviors
   - Ensure model names are correct for each provider

### Debug Mode

Enable detailed logging:

```rust
std::env::set_var("RUST_LOG", "debug");
tracing_subscriber::fmt::init();
```

## Related Files

- `src/services/fallback.rs` - Core fallback implementation
- `src/services/types/llm_service.rs` - Configuration types
- `src/services/types/llm_error.rs` - Error definitions
- `src/services/llm.rs` - Main LLM service integration
- `examples/fallback_example.rs` - Complete usage examples