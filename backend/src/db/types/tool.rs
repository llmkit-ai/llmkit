use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ToolRow {
    pub id: i64,
    pub name: String,
    pub tool_name: String,
    pub description: String,
    pub parameters: String,
    pub strict: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}