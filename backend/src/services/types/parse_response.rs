use hyper::HeaderMap;

#[derive(Debug, PartialEq)]
pub struct LlmApiResponseProps {
    pub response_content: String,
    pub raw_response: String,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub reasoning_tokens: Option<i64>
}

#[derive(Debug, PartialEq)]
pub struct LlmApiRequestProps {
    pub status: u16,
    pub body: String,
    pub method: String,
    pub url: String,
    pub headers: String,
}

impl LlmApiRequestProps {
    pub fn new(status: u16, body: String, method: String, url: String, headers: HeaderMap) -> Self {
        let headers = headers
            .iter()
            .map(|h| format!("{}:{:?}", h.0, h.1))
            .collect::<Vec<String>>()
            .join(" | ");
        LlmApiRequestProps {
            status,
            body,
            method,
            url,
            headers,
        }
    }
}
