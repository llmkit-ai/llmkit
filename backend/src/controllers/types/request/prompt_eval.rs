use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct CreateEvalTestRequest {
    pub prompt_id: i64,
    pub system_prompt_input: Option<Value>,
    pub user_prompt_input: Value,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEvalTestRequest {
    pub system_prompt_input: Option<Value>,
    pub user_prompt_input: String,
    pub name: String,
}
