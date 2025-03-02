#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    // HTTP/Network related errors
    #[error("HTTP error: {0}")]
    Http(reqwest::StatusCode),
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Eventsource cannot clone request: {0}")]
    EventSourceError(#[from] reqwest_eventsource::CannotCloneRequestError),
    #[error("Request timeout after {0} seconds")]
    Timeout(u64),
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),
    
    // Authentication errors
    #[error("Authentication error: {0}")]
    Auth(String),
    #[error("API key invalid or expired")]
    InvalidApiKey,
    #[error("Insufficient permissions for operation")]
    InsufficientPermissions,
    
    // Serialization/Deserialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Invalid UTF8 in chunk: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
    
    // Provider-specific errors
    #[error("Provider error: {0}")]
    Provider(String),
    #[error("Provider unavailable: {0}")]
    ProviderUnavailable(String),
    #[error("Provider quota exceeded")]
    ProviderQuotaExceeded,
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    #[error("Response from provider empty")]
    EmptyResponse,
    
    // Template/Prompt errors
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
    #[error("Missing system message")]
    MissingSystemMessage,
    #[error("Missing user message")]
    MissingUserMessage,
    #[error("Prompt exceeds token limit: {0}/{1}")]
    PromptTooLong(usize, usize),
    #[error("Content policy violation: {0}")]
    ContentPolicy(String),
    
    // Concurrency/Task errors
    #[error("MPSC Sender failed to send message in channel: {0}")]
    MpscSender(#[from] tokio::sync::mpsc::error::SendError<std::string::String>),
    #[error("JoinError in spawned tokio task: {0}")]
    TokioTaskJoin(#[from] tokio::task::JoinError),
    #[error("Task canceled")]
    TaskCanceled,
    
    // Logging/Metrics errors
    #[error("Missing Usage from chunk")]
    MissingUsage,
    #[error("DB Logging Error: {0}")]
    DbLoggingError(String),
    #[error("Failed to record metrics: {0}")]
    MetricsError(String),
    
    // Misc/Other errors
    #[error("Internal server error: {0}")]
    Internal(String),
    #[error("Feature not implemented: {0}")]
    NotImplemented(String),
    #[error("Resource not found: {0}")]
    NotFound(String),
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    // New errors for OpenRouter API library integration
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("Authentication error from OpenRouter: {0}")]
    AuthError(String),
    #[error("Header construction error: {0}")]
    HeaderError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Deserialization error: {0}")]
    DeserializationError(String)
}

impl From<openrouter_api::Error> for LlmError {
    fn from(err: openrouter_api::Error) -> Self {
        match err {
            openrouter_api::Error::HttpError(e) => LlmError::Network(e),
            openrouter_api::Error::ApiError { code, message, metadata: _ } => {
                // Map based on status code
                match code {
                    401 | 403 => LlmError::Auth(message),
                    404 => LlmError::NotFound(message),
                    429 => LlmError::RateLimit(message),
                    _ => LlmError::Provider(format!("API error ({}): {}", code, message))
                }
            },
            openrouter_api::Error::ConfigError(msg) => LlmError::InvalidConfig(msg),
            openrouter_api::Error::StructuredOutputNotSupported => LlmError::NotImplemented("Structured output not supported".to_string()),
            openrouter_api::Error::SchemaValidationError(msg) => LlmError::DeserializationError(msg),
            openrouter_api::Error::Unknown => LlmError::Internal("Unknown OpenRouter API error".to_string()),
        }
    }
}

#[derive(Debug)]
pub enum LlmStreamingError {
    StreamError(String),
    ParseError(String),
    ReceiverDropped,
}
