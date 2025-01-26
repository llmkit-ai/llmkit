#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelName {
   OpenAi(OpenAiModel),
   Anthropic(AnthropicModel), 
   Gemini(GeminiModel),
   Deepseek(DeepseekModel),
}

impl ModelName {
    pub fn provider(&self) -> String {
        match self {
            ModelName::OpenAi(_) => "openai",
            ModelName::Anthropic(_) => "anthropic", 
            ModelName::Gemini(_) => "gemini",
            ModelName::Deepseek(_) => "deepseek",
        }.to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenAiModel {
   Gpt4o202411,
   Gpt4oMini202407,
   O1202412,
   O1Mini202409,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnthropicModel {
   Claude35SonnetLatest,
   Claude35Sonnet20241022,
   Claude35HaikuLatest,
   Claude35Haiku20241022,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GeminiModel {
   Gemini20FlashThinkingExp0121,
   Gemini20FlashExp,
   Gemini15Flash,
   Gemini15Flash8b,
   Gemini15Pro,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeepseekModel {
   DeepseekChat,
   DeepseekReasoner,
}

impl From<String> for ModelName {
   fn from(value: String) -> Self {
       match value.as_str() {
           "gpt-4o-2024-11-20" => ModelName::OpenAi(OpenAiModel::Gpt4o202411),
           "gpt-4o-mini-2024-07-18" => ModelName::OpenAi(OpenAiModel::Gpt4oMini202407),
           "o1-2024-12-17" => ModelName::OpenAi(OpenAiModel::O1202412),
           "o1-mini-2024-09-12" => ModelName::OpenAi(OpenAiModel::O1Mini202409),

           "claude-3-5-sonnet-latest" => ModelName::Anthropic(AnthropicModel::Claude35SonnetLatest),
           "claude-3-5-sonnet-20241022" => ModelName::Anthropic(AnthropicModel::Claude35Sonnet20241022),
           "claude-3-5-haiku-latest" => ModelName::Anthropic(AnthropicModel::Claude35HaikuLatest),
           "claude-3-5-haiku-20241022" => ModelName::Anthropic(AnthropicModel::Claude35Haiku20241022),

           "gemini-2.0-flash-thinking-exp-01-21" => ModelName::Gemini(GeminiModel::Gemini20FlashThinkingExp0121),
           "gemini-2.0-flash-exp" => ModelName::Gemini(GeminiModel::Gemini20FlashExp),
           "gemini-1.5-flash" => ModelName::Gemini(GeminiModel::Gemini15Flash),
           "gemini-1.5-flash-8b" => ModelName::Gemini(GeminiModel::Gemini15Flash8b),
           "gemini-1.5-pro" => ModelName::Gemini(GeminiModel::Gemini15Pro),

           "deepseek-chat" => ModelName::Deepseek(DeepseekModel::DeepseekChat),
           "deepseek-reasoner" => ModelName::Deepseek(DeepseekModel::DeepseekReasoner),
           _ => unreachable!("Invalid ModelName")
       }
   }
}

impl From<ModelName> for String {
   fn from(value: ModelName) -> Self {
       match value {
           ModelName::OpenAi(model) => match model {
               OpenAiModel::Gpt4o202411 => "gpt-4o-2024-11-20",
               OpenAiModel::Gpt4oMini202407 => "gpt-4o-mini-2024-07-18",
               OpenAiModel::O1202412 => "o1-2024-12-17",
               OpenAiModel::O1Mini202409 => "o1-mini-2024-09-12",
           },
           ModelName::Anthropic(model) => match model {
               AnthropicModel::Claude35SonnetLatest => "claude-3-5-sonnet-latest",
               AnthropicModel::Claude35Sonnet20241022 => "claude-3-5-sonnet-20241022", 
               AnthropicModel::Claude35HaikuLatest => "claude-3-5-haiku-latest",
               AnthropicModel::Claude35Haiku20241022 => "claude-3-5-haiku-20241022",
           },
           ModelName::Gemini(model) => match model {
               GeminiModel::Gemini20FlashThinkingExp0121 => "gemini-2.0-flash-thinking-exp-01-21",
               GeminiModel::Gemini20FlashExp => "gemini-2.0-flash-exp",
               GeminiModel::Gemini15Flash => "gemini-1.5-flash",
               GeminiModel::Gemini15Flash8b => "gemini-1.5-flash-8b",
               GeminiModel::Gemini15Pro => "gemini-1.5-pro",
           },
           ModelName::Deepseek(model) => match model {
               DeepseekModel::DeepseekChat => "deepseek-chat",
               DeepseekModel::DeepseekReasoner => "deepseek-reasoner",
           },
       }.to_string()
   }
}
