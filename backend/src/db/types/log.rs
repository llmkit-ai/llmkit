use sqlx::prelude::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct LogRow {
    pub id: i64,
    pub prompt_id: Option<i64>,
    pub model_id: i64,
    pub status_code: Option<i64>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub reasoning_tokens: Option<i64>,
    pub response_data: Option<String>,
    pub request_body: Option<String>,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Clone, FromRow)]
pub struct LogRowModel {
    pub id: i64,
    pub prompt_id: Option<i64>,
    pub model_id: i64,
    pub model_name: String,
    pub provider_name: String,
    pub status_code: Option<i64>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub reasoning_tokens: Option<i64>,
    pub response_data: Option<String>,
    pub request_body: Option<String>,
    pub created_at: Option<chrono::NaiveDateTime>,
}
