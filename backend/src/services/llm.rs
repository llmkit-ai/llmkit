use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use tokio::sync::mpsc::Sender;
use tokio_retry::{Retry, strategy::{ExponentialBackoff, jitter}};
use std::time::Duration;
use dotenv::dotenv;
use tera::{Tera, Context as TeraContext};

use crate::common::types::models::ModelName;
use super::types::llm_props::LlmProps;


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
        Retry::spawn(retry_strategy, || self.send_request(ResponseFormat::Text))
            .await
    }

    pub async fn json<T: DeserializeOwned>(&self) -> Result<T, Error> {
        let retry_strategy = self.retry_strategy();
        Retry::spawn(retry_strategy, || async {
            let text = self.send_request(ResponseFormat::Json).await?;
            serde_json::from_str(&text).map_err(Into::into)
        })
        .await
    }

    pub async fn stream(&self, sender: Sender<String>) -> Result<String, Error> {
        // let (tx, mut rx) = mpsc::channel(10);
        todo!();
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
            ModelName::OpenAi(_) => self.build_openai_request(&model_name, format, &messages),
            ModelName::Anthropic(_) => self.build_anthropic_request(&model_name, format, &messages),
            ModelName::Gemini(_) => self.build_google_request(&model_name, format, &messages),
            ModelName::Deepseek(_) => self.build_deepseek_request(&model_name, format, &messages),
        }?;

        let response = request.send().await?;
        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            return Err(Error::Http(status));
        }

        match &self.props.model {
            ModelName::OpenAi(_) => Self::parse_openai_response(&text),
            ModelName::Anthropic(_) => Self::parse_anthropic_response(&text),
            ModelName::Gemini(_) => Self::parse_google_response(&text),
            ModelName::Deepseek(_) => Self::parse_deepseek_response(&text),
        }
    }

    fn build_openai_request(
        &self,
        model: &str,
        format: ResponseFormat,
        messages: &[Message],
    ) -> Result<reqwest::RequestBuilder, Error> {
        let api_key = std::env::var("OPENAI_API_KEY").map_err(|_| Error::Auth)?;
        let messages_json: Vec<_> = messages.iter()
            .map(|msg| json!({ "role": msg.role, "content": msg.content }))
            .collect();

        let mut body = serde_json::json!({
            "model": model,
            "messages": messages_json
        });

        if format == ResponseFormat::Json {
            body["response_format"] = serde_json::json!({ "type": "json_object" });
        }

        body["temperature"] = self.props.temperature.into();
        body["max_completion_tokens"] = self.props.max_tokens.into();
        
        Ok(self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&body))
    }

    fn build_anthropic_request(
        &self,
        model: &str,
        _format: ResponseFormat,
        messages: &[Message],
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
        });

        // Add optional parameters
        body["temperature"] = self.props.temperature.into();
        body["max_tokens"] = self.props.max_tokens.into();

        Ok(self.client
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

        Ok(self.client
            .post(&format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
                model
            ))
            .query(&[("key", api_key)])
            .json(&body))
    }

    fn build_deepseek_request(
        &self,
        model: &str,
        format: ResponseFormat,
        messages: &[Message],
    ) -> Result<reqwest::RequestBuilder, Error> {
        let api_key = std::env::var("DEEPSEEK_API_KEY").map_err(|_| Error::Auth)?;

        let messages_json: Vec<_> = messages.iter()
            .map(|msg| json!({ "role": msg.role, "content": msg.content }))
            .collect();

        let mut body = serde_json::json!({
            "model": model,
            "messages": messages_json
        });

        body["temperature"] = self.props.temperature.into();
        body["max_tokens"] = self.props.max_tokens.into();

        if format == ResponseFormat::Json {
            body["response_format"] = serde_json::json!({ "type": "json_object" });
        }
        
        Ok(self.client
            .post("https://api.deepseek.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&body))
    }

    // Response parsing functions remain the same as previous implementation
    fn parse_openai_response(text: &str) -> Result<String, Error> {
        #[derive(serde::Deserialize)]
        struct Response { choices: Vec<Choice> }
        #[derive(serde::Deserialize)]
        struct Choice { message: Message }
        #[derive(serde::Deserialize)]
        struct Message { content: String }

        let response: Response = serde_json::from_str(text)?;
        response.choices
            .first()
            .and_then(|c| Some(c.message.content.clone()))
            .ok_or(Error::Provider("Empty OpenAI response".into()))
    }

    fn parse_anthropic_response(text: &str) -> Result<String, Error> {
        #[derive(serde::Deserialize)]
        struct Response { content: Vec<Content> }
        #[derive(serde::Deserialize)]
        struct Content { text: String }

        let response: Response = serde_json::from_str(text)?;
        response.content
            .first()
            .and_then(|c| Some(c.text.clone()))
            .ok_or(Error::Provider("Empty Anthropic response".into()))
    }

    fn parse_google_response(text: &str) -> Result<String, Error> {
        #[derive(serde::Deserialize)]
        struct Response { candidates: Vec<Candidate> }
        #[derive(serde::Deserialize)]
        struct Candidate { content: Content }
        #[derive(serde::Deserialize)]
        struct Content { parts: Vec<Part> }
        #[derive(serde::Deserialize)]
        struct Part { text: String }

        let response: Response = serde_json::from_str(text)?;
        response.candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .and_then(|p| Some(p.text.clone()))
            .ok_or(Error::Provider("Empty Google response".into()))
    }

    fn parse_deepseek_response(text: &str) -> Result<String, Error> {
        #[derive(serde::Deserialize)]
        struct Response { choices: Vec<Choice> }
        #[derive(serde::Deserialize)]
        struct Choice { message: Message }
        #[derive(serde::Deserialize)]
        struct Message { content: String }

        let response: Response = serde_json::from_str(text)?;
        response.choices
            .first()
            .and_then(|c| Some(c.message.content.clone()))
            .ok_or(Error::Provider("Empty Deepseek response".into()))
    }

    fn retry_strategy(&self) -> impl Iterator<Item = Duration> {
        ExponentialBackoff::from_millis(100)
            .max_delay(Duration::from_secs(10))
            .map(jitter)
            .take(3)
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
        let role = line[PREFIX.len()..line.len()-SUFFIX.len()].trim().to_string();
        match role.as_str() {
            "system" | "user" | "assistant" => Some(role),
            _ => None
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use crate::common::types::models::{
        ModelName, OpenAiModel, AnthropicModel, GeminiModel, DeepseekModel
    };

    // Unit tests for response parsing
    #[test]
    fn test_openai_response_parsing() {
        let response = json!({
            "choices": [{
                "message": {
                    "content": "test response"
                }
            }]
        }).to_string();
        
        let result = Llm::parse_openai_response(&response);
        assert_eq!(result.unwrap(), "test response");
    }

    #[test]
    fn test_anthropic_response_parsing() {
        let response = json!({
            "content": [{
                "text": "test response"
            }]
        }).to_string();
        
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
        }).to_string();
        
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
        }).to_string();
        
        let result = Llm::parse_deepseek_response(&response);
        assert_eq!(result.unwrap(), "test response");
    }

    async fn create_test_props(model: ModelName) -> LlmProps {
        LlmProps {
            model,
            prompt: r#"<!-- role:system -->
                You are a helpful assistant
                <!-- role:user -->
                Hello, {{ name }}!"#.to_string(),
            context: json!({
                "name": "World",
            }),
            temperature: 0.5,
            max_tokens: 100,
            json_mode: false
        }
    }


    #[tokio::test]
    #[ignore]
    async fn test_openai_integration() {
        dotenv().ok();
        let props = create_test_props(ModelName::OpenAi(OpenAiModel::Gpt4oMini202407)).await;
        let llm = Llm::new(props).unwrap();
        
        // Test text response
        let text = llm.text().await.unwrap();
        assert!(text.contains("Hello"));
        
        // Test JSON response
        #[derive(serde::Deserialize)]
        struct TestResponse { message: String }
        
        let props = LlmProps {
            prompt: r#"<!-- role:system -->
                You must respond with valid JSON only
                <!-- role:user -->
                Return a JSON object with a 'message' field containing 'Hello in JSON'. Context: {{ message }}"#.to_string(),
            context: json!({
                "message": "Please respond with JSON",
                "response_format": {"type": "json_object"}
            }),
            ..create_test_props(ModelName::OpenAi(OpenAiModel::Gpt4oMini202407)).await
        };
        
        let llm = Llm::new(props).unwrap();
        let response: TestResponse = llm.json().await.unwrap();
        assert_eq!(response.message, "Hello in JSON");
    }

    #[tokio::test]
    #[ignore]
    async fn test_anthropic_integration() {
        dotenv().ok();
        let props = create_test_props(
            ModelName::Anthropic(AnthropicModel::Claude35Haiku20241022)
        ).await;
        let llm = Llm::new(props).unwrap();
        
        let text = llm.text().await.unwrap();
        assert!(text.contains("Hello"));
    }

    #[tokio::test]
    #[ignore]
    async fn test_google_integration() {
        dotenv().ok();
        
        // Test text response
        let props = create_test_props(ModelName::Gemini(GeminiModel::Gemini15Flash)).await;
        let llm = Llm::new(props).unwrap();
        let text = llm.text().await.unwrap();
        assert!(text.contains("Hello"));

        // Test JSON response
        #[derive(serde::Deserialize)]
        struct TestResponse { message: String }
        
        let props = LlmProps {
            prompt: r#"<!-- role:system -->
                You must respond with valid JSON only
                <!-- role:user -->
                Return a JSON object with a 'message' field containing 'Hello in JSON'. Context: {{ message }}"#.to_string(),
            context: json!({
                "message": "Please respond with JSON",
                "responseMimeType": "application/json"
            }),
            ..create_test_props(ModelName::Gemini(GeminiModel::Gemini15Flash)).await
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
        let props = create_test_props(ModelName::Deepseek(DeepseekModel::DeepseekChat)).await;
        let llm = Llm::new(props).unwrap();
        let text = llm.text().await.unwrap();
        assert!(text.contains("Hello"));

        // Test JSON response
        #[derive(serde::Deserialize)]
        struct TestResponse { 
            #[serde(rename = "content")]
            message: String 
        }
        
        let props = LlmProps {
            prompt: r#"<!-- role:system -->
                Respond with JSON containing a 'content' field
                <!-- role:user -->
                Return JSON with format: {"content": "<message>"}. Context: {{ message }}"#.to_string(),
            context: json!({
                "message": "Hello in JSON",
                "response_format": {"type": "json_object"}
            }),
            ..create_test_props(ModelName::Deepseek(DeepseekModel::DeepseekChat)).await
        };
        
        let llm = Llm::new(props).unwrap();
        let response: TestResponse = llm.json().await.unwrap();
        assert_eq!(response.message, "Hello in JSON");
    }

    #[tokio::test]
    async fn test_retry_logic() {
        let props = LlmProps {
            model: ModelName::OpenAi(OpenAiModel::Gpt4oMini202407),
            prompt: "test".to_string(),
            context: json!({}),
            temperature: 0.5,
            max_tokens: 100,
            json_mode: false
        };
        let llm = Llm::new(props).unwrap();
        
        let strategy = llm.retry_strategy();
        assert_eq!(strategy.count(), 3);
    }
}
