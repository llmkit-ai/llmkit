use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateToolRequest {
    pub name: String,
    pub tool_name: String,
    pub description: String,
    pub parameters: String,
    pub strict: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateToolRequest {
    pub name: String,
    pub tool_name: String,
    pub description: String,
    pub parameters: String,
    pub strict: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssociateToolPromptVersionRequest {
    pub tool_id: i64,
    pub prompt_version_id: i64,
}