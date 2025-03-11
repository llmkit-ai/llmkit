use serde::{Deserialize, Serialize};
use crate::db::types::tool::ToolRow;
use chrono::NaiveDateTime;

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolResponse {
    pub id: i64,
    pub name: String,
    pub tool_name: String,
    pub description: String,
    pub parameters: String,
    pub strict: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<ToolRow> for ToolResponse {
    fn from(tool: ToolRow) -> Self {
        Self {
            id: tool.id,
            name: tool.name,
            tool_name: tool.tool_name,
            description: tool.description,
            parameters: tool.parameters,
            strict: tool.strict,
            created_at: tool.created_at,
            updated_at: tool.updated_at,
        }
    }
}