use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct PromptRow {
    pub id: i64,
    pub key: String,
    pub system: String,
    pub user: String,
    pub model_id: i64,
    pub max_tokens: i64,
    pub temperature: f64,
    pub json_mode: bool,
    pub prompt_type: String,
    pub is_chat: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct PromptRowWithModel {
    pub id: i64,
    pub key: String,
    pub system: String,
    pub user: String,
    pub model_id: i64,
    pub max_tokens: i64,
    pub temperature: f64,
    pub json_mode: bool,
    pub json_schema: Option<String>,
    pub prompt_type: String,
    pub is_chat: bool,
    pub model_name: String,
    pub provider_name: String,
    pub provider_base_url: String,
    pub version_number: i64,
    pub version_id: i64,
    pub system_diff: Option<String>,
    pub user_diff: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}
