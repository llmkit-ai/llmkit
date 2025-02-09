use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum LlmModel {
   OpenAi(OpenAiModel),
   Anthropic(AnthropicModel), 
   Gemini(GeminiModel),
   Deepseek(DeepseekModel),
}

impl LlmModel {
    pub fn provider(&self) -> String {
        match self {
            LlmModel::OpenAi(_) => "openai",
            LlmModel::Anthropic(_) => "anthropic", 
            LlmModel::Gemini(_) => "gemini",
            LlmModel::Deepseek(_) => "deepseek",
        }.to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum OpenAiModel {
   Gpt4o202411,
   Gpt4oMini202407,
   O1202412,
   O1Mini202409,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum AnthropicModel {
   Claude35SonnetLatest,
   Claude35Sonnet20241022,
   Claude35HaikuLatest,
   Claude35Haiku20241022,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum GeminiModel {
   Gemini20FlashThinkingExp0121,
   Gemini20Flash001,
   Gemini20FlashLite,
   Gemini20ProExp0205,
   Gemini15Flash,
   Gemini15Flash8b,
   Gemini15Pro,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum DeepseekModel {
   DeepseekChat,
   DeepseekReasoner,
}

impl From<String> for LlmModel {
    fn from(value: String) -> Self {
        match value.as_str() {
            "gpt-4o-2024-11-20" => LlmModel::OpenAi(OpenAiModel::Gpt4o202411),
            "gpt-4o-mini-2024-07-18" => LlmModel::OpenAi(OpenAiModel::Gpt4oMini202407),
            "o1-2024-12-17" => LlmModel::OpenAi(OpenAiModel::O1202412),
            "o1-mini-2024-09-12" => LlmModel::OpenAi(OpenAiModel::O1Mini202409),

            "claude-3-5-sonnet-latest" => LlmModel::Anthropic(AnthropicModel::Claude35SonnetLatest),
            "claude-3-5-sonnet-20241022" => LlmModel::Anthropic(AnthropicModel::Claude35Sonnet20241022),
            "claude-3-5-haiku-latest" => LlmModel::Anthropic(AnthropicModel::Claude35HaikuLatest),
            "claude-3-5-haiku-20241022" => LlmModel::Anthropic(AnthropicModel::Claude35Haiku20241022),

            "gemini-2.0-flash-thinking-exp-01-21" => LlmModel::Gemini(GeminiModel::Gemini20FlashThinkingExp0121),
            "gemini-2.0-flash-001" => LlmModel::Gemini(GeminiModel::Gemini20Flash001),
            "gemini-2.0-flash-lite-preview-02-05" => LlmModel::Gemini(GeminiModel::Gemini20FlashLite),
            "gemini-2.0-pro-exp-02-05" => LlmModel::Gemini(GeminiModel::Gemini20ProExp0205),
            "gemini-1.5-flash" => LlmModel::Gemini(GeminiModel::Gemini15Flash),
            "gemini-1.5-flash-8b" => LlmModel::Gemini(GeminiModel::Gemini15Flash8b),
            "gemini-1.5-pro" => LlmModel::Gemini(GeminiModel::Gemini15Pro),

            "deepseek-chat" => LlmModel::Deepseek(DeepseekModel::DeepseekChat),
            "deepseek-reasoner" => LlmModel::Deepseek(DeepseekModel::DeepseekReasoner),
            _ => unreachable!("Invalid ModelName")
        }
    }
}

impl From<LlmModel> for String {
   fn from(value: LlmModel) -> Self {
       match value {
           LlmModel::OpenAi(model) => match model {
               OpenAiModel::Gpt4o202411 => "gpt-4o-2024-11-20",
               OpenAiModel::Gpt4oMini202407 => "gpt-4o-mini-2024-07-18",
               OpenAiModel::O1202412 => "o1-2024-12-17",
               OpenAiModel::O1Mini202409 => "o1-mini-2024-09-12",
           },
           LlmModel::Anthropic(model) => match model {
               AnthropicModel::Claude35SonnetLatest => "claude-3-5-sonnet-latest",
               AnthropicModel::Claude35Sonnet20241022 => "claude-3-5-sonnet-20241022", 
               AnthropicModel::Claude35HaikuLatest => "claude-3-5-haiku-latest",
               AnthropicModel::Claude35Haiku20241022 => "claude-3-5-haiku-20241022",
           },
           LlmModel::Gemini(model) => match model {
               GeminiModel::Gemini20FlashThinkingExp0121 => "gemini-2.0-flash-thinking-exp-01-21",
               GeminiModel::Gemini20Flash001 => "gemini-2.0-flash-001",
               GeminiModel::Gemini20FlashLite => "gemini-2.0-flash-lite-preview-02-05",
               GeminiModel::Gemini20ProExp0205 => "gemini-2.0-pro-exp-02-05",
               GeminiModel::Gemini15Flash => "gemini-1.5-flash",
               GeminiModel::Gemini15Flash8b => "gemini-1.5-flash-8b",
               GeminiModel::Gemini15Pro => "gemini-1.5-pro",
           },
           LlmModel::Deepseek(model) => match model {
               DeepseekModel::DeepseekChat => "deepseek-chat",
               DeepseekModel::DeepseekReasoner => "deepseek-reasoner",
           },
       }.to_string()
   }
}
