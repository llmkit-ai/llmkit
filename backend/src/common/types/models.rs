use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum LlmApiProvider {
    Openrouter,
    OpenAi,
    Azure,

    // TODO: Will support in future with more refined SDK
    // Anthropic,
    // Gemini,
    // Deepseek,
}

impl From<String> for LlmApiProvider {
    fn from(value: String) -> Self {
        match value.as_str() {
            "openrouter" => LlmApiProvider::Openrouter,
            "openai" => LlmApiProvider::OpenAi,
            "azure" => LlmApiProvider::Azure,
            _ => unreachable!("Invalid Provider"),
        }
    }
}

impl From<LlmApiProvider> for String {
    fn from(value: LlmApiProvider) -> Self {
        match value {
            LlmApiProvider::Openrouter => "openrouter".to_string(),
            LlmApiProvider::OpenAi => "openai".to_string(),
            LlmApiProvider::Azure => "azure".to_string(),
        }
    }
}
