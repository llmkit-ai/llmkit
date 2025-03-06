use serde::{Deserialize, Serialize};
use crate::db::types::tool::{ToolWithVersion, ToolVersionRow};
use chrono::NaiveDateTime;

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolResponse {
    pub id: i64,
    pub name: String,
    pub current_tool_version_id: Option<i64>,
    pub version_number: i64,
    pub tool_name: String,
    pub description: String,
    pub parameters: String,
    pub strict: bool,
    pub version_id: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<ToolWithVersion> for ToolResponse {
    fn from(tool: ToolWithVersion) -> Self {
        Self {
            id: tool.id,
            name: tool.name,
            current_tool_version_id: tool.current_tool_version_id,
            version_number: tool.version_number,
            tool_name: tool.tool_name,
            description: tool.description,
            parameters: tool.parameters,
            strict: tool.strict,
            version_id: tool.version_id,
            created_at: tool.created_at,
            updated_at: tool.updated_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolVersionResponse {
    pub id: i64,
    pub tool_id: i64,
    pub version_number: i64,
    pub tool_name: String,
    pub description: String,
    pub parameters: String,
    pub strict: bool,
    pub created_at: NaiveDateTime,
}

impl From<ToolVersionRow> for ToolVersionResponse {
    fn from(version: ToolVersionRow) -> Self {
        Self {
            id: version.id,
            tool_id: version.tool_id,
            version_number: version.version_number,
            tool_name: version.tool_name,
            description: version.description,
            parameters: version.parameters,
            strict: version.strict,
            created_at: version.created_at,
        }
    }
}