use serde::Serialize;
use crate::db::types::log::LogRowModel;

#[derive(Debug, Serialize)]
pub struct ApiLogResponse {
    pub id: i64,
    pub prompt_id: Option<i64>,
    pub model_id: i64,
    pub model_name: String,
    pub response_data: Option<String>,
    pub status_code: Option<i64>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub request_body: Option<String>,
    pub provider_response_id: String,
    pub created_at: String
}

impl From<LogRowModel> for ApiLogResponse {
    fn from(log: LogRowModel) -> Self {
        ApiLogResponse {
            id: log.id,
            prompt_id: log.prompt_id,
            model_id: log.model_id,
            model_name: log.model_name,
            response_data: log.response_data,
            status_code: log.status_code,
            input_tokens: log.input_tokens,
            output_tokens: log.output_tokens,
            request_body: log.request_body,
            provider_response_id: log.provider_response_id,
            created_at: log.created_at.map(|v| v.to_string()).unwrap_or_default()
        }
    }
}

#[derive(Serialize)]
pub struct ApiLogCountResponse {
    pub count: i64,
}

