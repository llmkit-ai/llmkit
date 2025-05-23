use std::time::Duration;

use anyhow::Result;
use tokio::sync::mpsc::Sender;
use tokio_retry::{
    strategy::{jitter, ExponentialBackoff},
    Retry,
};
use tracing;

use super::{
    providers::openrouter::OpenrouterProvider,
    types::{
        llm_service::LlmServiceRequest,
        llm_error::{LlmError, LlmStreamingError},
    },
};
use crate::{common::types::{chat_response::{LlmServiceChatCompletionChunk, LlmServiceChatCompletionResponse}, models::LlmApiProvider}, db::logs::LogRepository};

pub struct Llm {
    props: LlmServiceRequest,
    db_log: LogRepository,
    run_id: Option<String>,
}

impl Llm {
    pub fn new(props: LlmServiceRequest, db_log: LogRepository) -> Self {
        Llm { props, db_log, run_id: None }
    }

    pub fn new_with_run_id(props: LlmServiceRequest, db_log: LogRepository, run_id: String) -> Self {
        Llm { props, db_log, run_id: Some(run_id) }
    }

    fn retry_strategy(&self) -> impl Iterator<Item = Duration> {
        ExponentialBackoff::from_millis(100)
            .max_delay(Duration::from_secs(3))
            .map(jitter)
            .take(5)
    }

    pub async fn text(&self) -> Result<(LlmServiceChatCompletionResponse, i64), LlmError> {
        let retry_strategy = self.retry_strategy();
        Retry::spawn(retry_strategy, || self.send_request()).await
    }

    pub async fn json(&self) -> Result<(LlmServiceChatCompletionResponse, i64), LlmError> {
        let retry_strategy = self.retry_strategy();
        Retry::spawn(retry_strategy, || async {
            let res = self.send_request().await?;

            if let Some(c) = res.0.choices.first() {
                // We don't need to validate the tool response
                if c.message.role == "tool" {
                    return Ok(res);
                }

                let content = match &c.message.content {
                    Some(c) => c.to_string(),
                    None => return Err(LlmError::MissingAssistantContent)
                };

                // if we have a JSON schema available lets use it
                // Otherwise just make sure it's valid JSON and return
                match &self.props.request.response_format {
                    Some(rf) => {
                        match &rf.json_schema {
                            Some(js) => {
                                let is_valid = &self.validate_schema(&content, &js.schema)?;
                                if !is_valid {
                                    tracing::error!("The schema was not valid");
                                    return Err(LlmError::InvalidJsonSchema);
                                }
                            },
                            None => {
                                let _json: serde_json::Value = serde_json::from_str(&content)?;
                            } 
                        }
                    },
                    None => unreachable!("Encountered a situation where we don't have a response_format in JSON mode")
                }
            }

            Ok(res)
        })
        .await
    }

    fn validate_schema(&self, response: &str, schema: &serde_json::Value) -> Result<bool, LlmError> {
        let response_json: serde_json::Value = serde_json::from_str(&response)?;
        let is_valid = jsonschema::is_valid(&schema, &response_json);

        if !is_valid {
            return Ok(false);
        }

        Ok(true)
    }

    pub async fn stream(
        &self,
        tx: Sender<Result<LlmServiceChatCompletionChunk, LlmStreamingError>>,
    ) -> Result<(LlmServiceChatCompletionResponse, i64), LlmError> {
        if self.props.request.response_format.is_some() {
            tracing::info!("Json mode not supported in chat mode");
            return Err(LlmError::UnsupportedMode(
                "Json".to_string(),
                "Chat".to_string(),
            ));
        }

        Ok(self.send_request_stream(tx).await?)
    }

    async fn send_request(&self) -> Result<(LlmServiceChatCompletionResponse, i64), LlmError> {
        // Initialize variables to capture data even in error cases
        let mut input_tokens = None;
        let mut output_tokens = None;
        let reasoning_tokens = None;
        let mut raw_response: Option<String> = None;
        let mut status = Some(500); // Default to error status

        // Serialize the request for logging
        let request_body = serde_json::to_string(&self.props)
            .map_err(|e| LlmError::SerializationError(e.to_string()))?;

        // Execute request and capture result
        let result = match &self.props.provider {
            LlmApiProvider::Openrouter => {
                let provider = OpenrouterProvider::new(&self.props, false)?;
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
                self.run_id.as_deref(),
            )
            .await?;

        // Return the original result but with the correct log_id
        match exec_result {
            Ok(r) => Ok((r, log_id)),
            Err(e) => Err(e),
        }
    }

    async fn send_request_stream(
        &self,
        tx: Sender<Result<LlmServiceChatCompletionChunk, LlmStreamingError>>,
    ) -> Result<(LlmServiceChatCompletionResponse, i64), LlmError> {
        // Initialize variables to capture data even in error cases
        let mut input_tokens = None;
        let mut output_tokens = None;
        let reasoning_tokens = None;
        let mut raw_response: Option<String> = None;
        let mut status = Some(500); // Default to error status

        // Serialize the request for logging
        let request_body = serde_json::to_string(&self.props)
            .map_err(|e| LlmError::SerializationError(e.to_string()))?;

        // Check json mode before making the request
        if self.props.request.response_format.is_some() {
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
                self.run_id.as_deref(),
            )
            .await?;

            return Err(error);
        }

        // Execute request and capture result
        let result = match &self.props.provider {
            LlmApiProvider::Openrouter => {
                let provider = OpenrouterProvider::new(&self.props, true)?;
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
                self.run_id.as_deref(),
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
        run_id: Option<&str>,
    ) -> Result<i64, LlmError> {
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
                provider_response_id,
                run_id,
            )
            .await
            .map_err(|e| LlmError::DbLoggingError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // This struct represents a test version of Llm that we can use for unit testing the validate_schema method
    struct TestLlm {}

    impl TestLlm {
        // Create a simplified version of validate_schema for testing
        fn validate_schema(&self, response: &str, schema: &str) -> Result<bool, LlmError> {
            let response_json: serde_json::Value = serde_json::from_str(&response)?;
            let schema_json: serde_json::Value = serde_json::from_str(&schema)?;
            let is_valid = jsonschema::is_valid(&schema_json, &response_json);

            if !is_valid {
                return Ok(false);
            }

            Ok(true)
        }
    }

    #[test]
    fn test_validate_schema_valid_response() {
        let llm = TestLlm {};
        
        // Simple schema requiring a string field named "test"
        let schema = r#"{
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "required": ["test"],
            "properties": {
                "test": {"type": "string"}
            }
        }"#;
        
        // Valid response matching the schema
        let response = r#"{"test": "hello world"}"#;
        
        let result = llm.validate_schema(response, schema);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_validate_schema_invalid_response() {
        let llm = TestLlm {};
        
        // Simple schema requiring a string field named "test"
        let schema = r#"{
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "required": ["test"],
            "properties": {
                "test": {"type": "string"}
            }
        }"#;
        
        // Invalid response missing required field
        let response = r#"{"other_field": "hello world"}"#;
        
        let result = llm.validate_schema(response, schema);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_validate_schema_wrong_type() {
        let llm = TestLlm {};
        
        // Simple schema requiring a string field named "test"
        let schema = r#"{
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "required": ["test"],
            "properties": {
                "test": {"type": "string"}
            }
        }"#;
        
        // Invalid response with wrong type for field
        let response = r#"{"test": 123}"#;
        
        let result = llm.validate_schema(response, schema);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_validate_schema_product_review() {
        let llm = TestLlm {};
        
        // Complex schema for product review
        let schema = r#"{
          "$schema": "http://json-schema.org/draft-07/schema#",
          "type": "object",
          "required": [
            "key_features",
            "pros",
            "cons",
            "target_users",
            "rating",
            "summary"
          ],
          "properties": {
            "key_features": {
              "type": "array",
              "items": {
                "type": "string"
              }
            },
            "pros": {
              "type": "array",
              "items": {
                "type": "string"
              }
            },
            "cons": {
              "type": "array",
              "items": {
                "type": "string"
              }
            },
            "target_users": {
              "type": "array",
              "items": {
                "type": "string"
              }
            },
            "rating": {
              "type": "number",
              "minimum": 1,
              "maximum": 10
            },
            "summary": {
              "type": "string"
            }
          }
        }"#;
        
        // Valid response matching the product review schema
        let response = r#"{
            "key_features": ["Feature 1", "Feature 2", "Feature 3"],
            "pros": ["Pro 1", "Pro 2"],
            "cons": ["Con 1"],
            "target_users": ["User type 1", "User type 2"],
            "rating": 8.5,
            "summary": "This is a great product overall."
        }"#;
        
        let result = llm.validate_schema(response, schema);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_validate_schema_product_review_invalid() {
        let llm = TestLlm {};
        
        // Complex schema for product review
        let schema = r#"{
          "$schema": "http://json-schema.org/draft-07/schema#",
          "type": "object",
          "required": [
            "key_features",
            "pros",
            "cons",
            "target_users",
            "rating",
            "summary"
          ],
          "properties": {
            "key_features": {
              "type": "array",
              "items": {
                "type": "string"
              }
            },
            "pros": {
              "type": "array",
              "items": {
                "type": "string"
              }
            },
            "cons": {
              "type": "array",
              "items": {
                "type": "string"
              }
            },
            "target_users": {
              "type": "array",
              "items": {
                "type": "string"
              }
            },
            "rating": {
              "type": "number",
              "minimum": 1,
              "maximum": 10
            },
            "summary": {
              "type": "string"
            }
          }
        }"#;
        
        // Invalid response: missing required fields and invalid rating value
        let response = r#"{
            "key_features": ["Feature 1", "Feature 2"],
            "pros": ["Pro 1", "Pro 2"],
            "rating": 11,
            "summary": "This is a great product overall."
        }"#;
        
        let result = llm.validate_schema(response, schema);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_validate_schema_invalid_json() {
        let llm = TestLlm {};
        
        // Simple schema
        let schema = r#"{
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "test": {"type": "string"}
            }
        }"#;
        
        // Invalid JSON response
        let response = r#"{"test": "unclosed string"#;
        
        let result = llm.validate_schema(response, schema);
        assert!(result.is_err());
    }
}
