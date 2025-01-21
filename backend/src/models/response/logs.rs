use serde::Serialize;
use crate::db::logs::LlmApiTrace;

#[derive(Debug, Serialize)]
pub struct ApiTraceResponse {
    pub id: i64,
    pub prompt_id: Option<i64>,
    pub provider: String,
    pub model: String,
    pub request_data: String,
    pub response_data: Option<String>,
    pub status_code: Option<i64>,
    pub latency_ms: Option<i64>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
}

impl From<LlmApiTrace> for ApiTraceResponse {
    fn from(trace: LlmApiTrace) -> Self {
        ApiTraceResponse {
            id: trace.id,
            prompt_id: trace.prompt_id,
            provider: trace.provider,
            model: trace.model,
            request_data: trace.request_data,
            response_data: trace.response_data,
            status_code: trace.status_code,
            latency_ms: trace.latency_ms,
            input_tokens: trace.input_tokens,
            output_tokens: trace.output_tokens,
            error_code: trace.error_code,
            error_message: trace.error_message
        }
    }
}
