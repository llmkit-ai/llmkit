
#[derive(Debug)]
pub enum LlmStreamingError {
    StreamError(String),
    ParseError(String),
    ReceiverDropped,
}

