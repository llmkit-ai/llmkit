use serde::Serialize;

use crate::db::types::log::LogRow;

#[derive(Debug, Serialize)]
pub struct ApiTraceResponse {
    pub id: i64,
    pub prompt_id: Option<i64>,
    pub model_id: i64,
    pub response_data: Option<String>,
    pub status_code: Option<i64>,
    pub latency_ms: Option<i64>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub request_body: Option<String>,
    pub request_method: Option<String>,
    pub request_url: Option<String>,
    pub request_headers: Option<String>,
}

impl From<LogRow> for ApiTraceResponse {
    fn from(trace: LogRow) -> Self {
        ApiTraceResponse {
            id: trace.id,
            prompt_id: trace.prompt_id,
            model_id: trace.model_id,
            response_data: trace.response_data,
            status_code: trace.status_code,
            latency_ms: trace.latency_ms,
            input_tokens: trace.input_tokens,
            output_tokens: trace.output_tokens,
            request_body: trace.request_body,
            request_method: trace.request_method,
            request_url: trace.request_url,
            request_headers: trace.request_headers,
        }
    }
}
