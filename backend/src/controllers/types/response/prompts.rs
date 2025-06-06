use serde::Serialize;

use crate::db::types::prompt::PromptRowWithModel;
use crate::controllers::types::response::tools::ToolResponse;


// GET PROMPT RESPONSE
#[derive(Debug, Serialize)]
pub struct PromptResponse {
    pub id: i64,
    pub key: String,
    pub system: String,
    pub user: Option<String>,
    pub model: String,
    pub model_id: i64,
    pub provider: String,
    pub max_tokens: i64,
    pub temperature: f64,
    pub json_mode: bool,
    pub json_schema: Option<String>,
    pub prompt_type: String,
    pub is_chat: bool,
    pub version_id: i64,
    pub version_number: i64,
    pub system_version_diff: Option<String>,
    pub user_version_diff: Option<String>,
    pub updated_at: String,
    pub tools: Vec<ToolResponse>,
    pub supports_json: bool,
    pub supports_json_schema: bool,
    pub supports_tools: bool,
    pub is_reasoning: bool,
    pub reasoning_effort: Option<String>
}


impl From<PromptRowWithModel> for PromptResponse {
    fn from(prompt: PromptRowWithModel) -> Self {
        PromptResponse {
            id: prompt.id,
            key: prompt.key,
            system: prompt.system,
            user: prompt.user,
            model: prompt.model_name.into(),
            model_id: prompt.model_id,
            provider: prompt.provider_name.into(),
            max_tokens: prompt.max_tokens,
            temperature: prompt.temperature,
            json_mode: prompt.json_mode,
            json_schema: prompt.json_schema,
            prompt_type: prompt.prompt_type,
            is_chat: prompt.is_chat,
            version_id: prompt.version_id,
            version_number: prompt.version_number,
            system_version_diff: prompt.system_diff,
            user_version_diff: prompt.user_diff,
            updated_at: prompt.updated_at.to_string(),
            tools: Vec::new(), // Initialize with empty vector, will be filled separately
            supports_json: prompt.supports_json,
            supports_json_schema: prompt.supports_json_schema,
            supports_tools: prompt.supports_tools,
            is_reasoning: prompt.is_reasoning,
            reasoning_effort: prompt.reasoning_effort
        }
    }
}
