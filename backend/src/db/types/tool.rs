use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ToolRow {
    pub id: i64,
    pub name: String,
    pub current_tool_version_id: Option<i64>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ToolVersionRow {
    pub id: i64,
    pub tool_id: i64,
    pub version_number: i64,
    pub tool_name: String,
    pub description: String,
    pub parameters: String,
    pub strict: bool,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ToolWithVersion {
    pub id: i64,
    pub name: String,
    pub current_tool_version_id: Option<i64>,
    pub version_number: i64,
    pub tool_name: String,
    pub description: String,
    pub parameters: String,
    pub strict: bool,
    pub version_id: i64,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}