use serde::{Deserialize, Serialize};

use crate::db::types::{log::LogRow, prompt::PromptWithModel};


// GET PROMPT RESPONSE
#[derive(Debug, Serialize)]
pub struct PromptResponse {
    pub id: i64,
    pub key: String,
    pub prompt: String,
    pub model: String,
    pub model_id: i64,
    pub provider: String,
    pub max_tokens: i64,
    pub temperature: f64,
    pub json_mode: bool
}


impl From<PromptWithModel> for PromptResponse {
    fn from(prompt: PromptWithModel) -> Self {
        PromptResponse {
            id: prompt.id,
            key: prompt.key,
            prompt: prompt.prompt,
            model: prompt.model_name.into(),
            model_id: prompt.model_id,
            provider: prompt.provider,
            max_tokens: prompt.max_tokens,
            temperature: prompt.temperature,
            json_mode: prompt.json_mode,
        }
    }
}



// PROMPT EXECUTION RESPONSE
#[derive(Debug, Serialize, Deserialize)]
pub struct PromptExecutionResponse {
    pub content: String,
    pub trace: ApiTraceResponse,
}

#[derive(Debug, Serialize, Deserialize)]
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

impl PromptExecutionResponse {
    pub fn from_log_row(content: String, log_row: LogRow) -> Self {
        PromptExecutionResponse {
            content,
            trace: ApiTraceResponse::from(log_row),
        }
    }
}
