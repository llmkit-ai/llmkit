use sqlx::prelude::FromRow;


#[derive(Debug, Clone, FromRow)]
pub struct LogRow {
    pub id: i64,
    pub prompt_id: Option<i64>,
    pub model_id: i64,
    pub request_data: String,
    pub response_data: Option<String>,
    pub status_code: Option<i64>,
    pub latency_ms: Option<i64>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub created_at: Option<chrono::NaiveDateTime>,
}

