use serde::{Deserialize, Serialize};
use openrouter_api::{types::chat::ChatCompletionResponse, ChatCompletionChunk};

/// Chat completion response.
#[derive(Debug, Deserialize, Serialize)]
pub struct LlmServiceChatCompletionResponse {
    pub id: String,
    pub choices: Vec<LlmServiceChatCompletionResponseChoice>,
    pub created: i64,
    pub model: String,
    pub object: String,
    pub usage: Option<LlmServiceChatCompletionResponseUsage>,
}

/// A choice returned by the chat API.
#[derive(Debug, Deserialize, Serialize)]
pub struct LlmServiceChatCompletionResponseChoice {
    pub index: u32,
    pub message: LlmServiceChatCompletionResponseMessage,
    pub finish_reason: Option<String>,
    #[serde(rename = "native_finish_reason")]
    pub native_finish_reason: Option<String>,
}

/// Usage data returned from the API.
#[derive(Debug, Deserialize, Serialize)]
pub struct LlmServiceChatCompletionResponseUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LlmServiceChatCompletionResponseMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    // Optionally include tool_calls when the assistant message contains a tool call.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<LlmServiceChatCompletionResponseToolCall>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LlmServiceChatCompletionResponseToolCall {
    /// A unique identifier for the tool call.
    pub id: Option<String>,
    /// The index of the tool call in the list of tool calls
    pub index: u32,
    /// The type of call. When streaming, the first chunk only will contain "function".
    #[serde(rename = "type")]
    pub kind: Option<String>,
    /// The details of the function call, including its function name and arguments.
    #[serde(rename = "function")]
    pub function_call: LlmServiceChatCompletionResponseFunctionCall,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LlmServiceChatCompletionResponseFunctionCall {
    /// The name of the function to call.
    pub name: Option<String>,
    /// A JSON string representing the arguments for the function call.
    pub arguments: String,
}

impl LlmServiceChatCompletionResponse {
    /// Creates a new LlmServiceChatCompletionResponse with simplified parameters.
    /// Constructs a single choice with the given message content.
    /// Useful for handling streamed responses which are typically simpler.
    pub fn new_streamed(
        id: String,
        message_content: String,
        model: String,
        created: i64,
        prompt_tokens: Option<u32>,
        completion_tokens: Option<u32>,
        total_tokens: Option<u32>,
    ) -> Self {
        // Create a single choice with the message content
        let choice = LlmServiceChatCompletionResponseChoice {
            index: 0,
            message: LlmServiceChatCompletionResponseMessage {
                role: "assistant".to_string(),
                content: message_content,
                name: None,
                tool_calls: None,
            },
            finish_reason: Some("stop".to_string()),
            native_finish_reason: None,
        };
        
        // Construct usage if all token counts are provided
        let usage = match (prompt_tokens, completion_tokens, total_tokens) {
            (Some(prompt), Some(completion), Some(total)) => Some(LlmServiceChatCompletionResponseUsage {
                prompt_tokens: prompt,
                completion_tokens: completion,
                total_tokens: total,
            }),
            _ => None,
        };
        
        Self {
            id,
            choices: vec![choice],
            created,
            model,
            usage,
            object: "chat.completion".to_string(),
        }
    }
}


impl From<ChatCompletionResponse> for LlmServiceChatCompletionResponse {
    fn from(value: ChatCompletionResponse) -> Self {
        LlmServiceChatCompletionResponse {
            id: value.id,
            choices: value.choices.into_iter().enumerate().map(|(index, choice)| {
                LlmServiceChatCompletionResponseChoice {
                    index: index as u32,
                    message: LlmServiceChatCompletionResponseMessage {
                        role: choice.message.role,
                        content: choice.message.content,
                        name: choice.message.name,
                        tool_calls: choice.message.tool_calls.map(|tool_calls| {
                            tool_calls.into_iter().map(|tool_call| {
                                LlmServiceChatCompletionResponseToolCall {
                                    id: tool_call.id,
                                    index: tool_call.index,
                                    kind: tool_call.kind,
                                    function_call: LlmServiceChatCompletionResponseFunctionCall {
                                        name: tool_call.function_call.name,
                                        arguments: tool_call.function_call.arguments,
                                    },
                                }
                            }).collect()
                        }),
                    },
                    finish_reason: choice.finish_reason,
                    native_finish_reason: choice.native_finish_reason,
                }
            }).collect(),
            created: value.created,
            model: value.model,
            object: "chat.completion".to_string(),
            usage: value.usage.map(|usage| {
                LlmServiceChatCompletionResponseUsage {
                    prompt_tokens: usage.prompt_tokens,
                    completion_tokens: usage.completion_tokens,
                    total_tokens: usage.total_tokens,
                }
            }),
        }
    }
}


#[derive(Debug, Deserialize, Serialize)]
pub struct LlmServiceChatCompletionChunk {
    pub id: String,
    pub choices: Vec<LlmServiceChoiceStream>,
    pub usage: Option<LlmServiceUsage>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LlmServiceChoiceStream {
    pub index: u32,
    pub delta: LlmServiceStreamDelta,
    pub finish_reason: Option<String>,
    pub native_finish_reason: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LlmServiceStreamDelta {
    pub role: Option<String>,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<LlmServiceChatCompletionResponseToolCall>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LlmServiceUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

impl LlmServiceChatCompletionChunk {
    /// Creates a special "DONE" sentinel chunk to signify that the stream is complete.
    pub fn done_sentinel(id: String) -> Self {
        LlmServiceChatCompletionChunk {
            id,
            choices: vec![LlmServiceChoiceStream {
                index: 0,
                delta: LlmServiceStreamDelta {
                    role: Some("assistant".to_string()),
                    content: Some("[DONE]".to_string()),
                    tool_calls: None,
                },
                finish_reason: Some("stop".to_string()),
                native_finish_reason: Some("stop".to_string()),
            }],
            usage: None,
        }
    }
    
    /// Checks if this chunk is a "DONE" sentinel.
    pub fn is_done_sentinel(&self) -> bool {
        self.choices.iter().any(|choice| 
            choice.delta.content == Some("[DONE]".to_string()) && 
            choice.finish_reason.as_deref() == Some("stop")
        )
    }
}

impl From<ChatCompletionChunk> for LlmServiceChatCompletionChunk {
    fn from(chunk: ChatCompletionChunk) -> Self {
        LlmServiceChatCompletionChunk {
            id: chunk.id,
            choices: chunk.choices.into_iter().map(|choice| {
                LlmServiceChoiceStream {
                    index: choice.index,
                    delta: LlmServiceStreamDelta {
                        role: choice.delta.role,
                        content: choice.delta.content,
                        tool_calls: choice.delta.tool_calls
                            .map(|tc|
                                tc.into_iter().map(|tool_call| {
                                    LlmServiceChatCompletionResponseToolCall {
                                        id: tool_call.id,
                                        index: tool_call.index,
                                        kind: tool_call.kind,
                                        function_call: LlmServiceChatCompletionResponseFunctionCall {
                                            name: tool_call.function_call.name,
                                            arguments: tool_call.function_call.arguments,
                                        },
                                    }
                                }).collect()
                            )
                        
                    },
                    finish_reason: choice.finish_reason,
                    native_finish_reason: choice.native_finish_reason,
                }
            }).collect(),
            usage: chunk.usage.map(|usage| {
                LlmServiceUsage {
                    prompt_tokens: usage.prompt_tokens,
                    completion_tokens: usage.completion_tokens,
                    total_tokens: usage.total_tokens,
                }
            }),
        }
    }
}
