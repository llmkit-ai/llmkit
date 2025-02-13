use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum LlmApiProvider {
    OpenAi,
    Anthropic,
    Gemini,
    Deepseek,
    Azure,
}

impl From<String> for LlmApiProvider {
    fn from(value: String) -> Self {
        match value.as_str() {
            "openai" => LlmApiProvider::OpenAi,
            "anthropic" => LlmApiProvider::Anthropic,
            "gemini" => LlmApiProvider::Gemini,
            "deepseek" => LlmApiProvider::Deepseek,
            "azure" => LlmApiProvider::Azure,
            _ => unreachable!("Invalid Provider"),
        }
    }
}

impl From<LlmApiProvider> for String {
    fn from(value: LlmApiProvider) -> Self {
        match value {
            LlmApiProvider::OpenAi => "openai".to_string(),
            LlmApiProvider::Anthropic => "anthropic".to_string(),
            LlmApiProvider::Gemini => "gemini".to_string(),
            LlmApiProvider::Deepseek => "deepseek".to_string(),
            LlmApiProvider::Azure => "azure".to_string(),
        }.to_string()
    }
}
