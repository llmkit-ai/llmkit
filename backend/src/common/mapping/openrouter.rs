use crate::common::types::{chat_request::{
    ChatCompletionRequestFunctionCall, ChatCompletionRequestFunctionDescription,
    ChatCompletionRequestJsonSchema, ChatCompletionRequestResponseFormat,
    ChatCompletionRequestTool, ChatCompletionRequestToolCall,
}, chat_response::{LlmServiceChatCompletionChunk, LlmServiceChatCompletionResponse, LlmServiceChatCompletionResponseChoice, LlmServiceChatCompletionResponseFunctionCall, LlmServiceChatCompletionResponseMessage, LlmServiceChatCompletionResponseToolCall, LlmServiceChatCompletionResponseUsage, LlmServiceChoiceStream, LlmServiceStreamDelta, LlmServiceUsage}};
use openrouter_api::{models::tool::{FunctionCall, ToolCall}, ChatCompletionChunk, ChatCompletionResponse};

impl From<ChatCompletionRequestResponseFormat> for openrouter_api::types::chat::ResponseFormat {
    fn from(value: ChatCompletionRequestResponseFormat) -> Self {
        openrouter_api::types::chat::ResponseFormat {
            format_type: value.format_type,
            json_schema: value.json_schema.map(|js| js.into()),
        }
    }
}

impl From<ChatCompletionRequestJsonSchema> for openrouter_api::types::chat::JsonSchema {
    fn from(value: ChatCompletionRequestJsonSchema) -> Self {
        openrouter_api::types::chat::JsonSchema {
            name: value.name,
            strict: value.strict,
            schema: value.schema,
        }
    }
}

impl From<openrouter_api::models::tool::Tool> for ChatCompletionRequestTool {
    fn from(value: openrouter_api::models::tool::Tool) -> Self {
        match value {
            openrouter_api::models::tool::Tool::Function { function } => {
                let description = ChatCompletionRequestFunctionDescription {
                    name: function.name,
                    description: function.description,
                    parameters: function.parameters,
                    strict: None
                };

                ChatCompletionRequestTool::Function {
                    function: description,
                }
            }
        }
    }
}

impl From<ChatCompletionRequestTool> for openrouter_api::models::tool::Tool {
    fn from(value: ChatCompletionRequestTool) -> Self {
        match value {
            ChatCompletionRequestTool::Function { function } => {
                let description = openrouter_api::models::tool::FunctionDescription {
                    name: function.name,
                    description: function.description,
                    parameters: function.parameters,
                };

                openrouter_api::models::tool::Tool::Function {
                    function: description,
                }
            }
        }
    }
}

// Impl for Opernrouter SDK
impl Into<ToolCall> for ChatCompletionRequestToolCall {
    fn into(self) -> ToolCall {
        ToolCall {
            id: Some(self.id),
            index: None,
            kind: Some(self.kind),
            function_call: self.function_call.into(),
        }
    }
}

// Impl for Opernrouter SDK
impl Into<FunctionCall> for ChatCompletionRequestFunctionCall {
    fn into(self) -> FunctionCall {
        FunctionCall {
            name: Some(self.name),
            arguments: self.arguments,
        }
    }
}

impl From<ChatCompletionResponse> for LlmServiceChatCompletionResponse {
    fn from(value: ChatCompletionResponse) -> Self {
        LlmServiceChatCompletionResponse {
            id: value.id,
            choices: value.choices.into_iter().enumerate().map(|(index, choice)| {
                LlmServiceChatCompletionResponseChoice { index: index as u32,
                    message: LlmServiceChatCompletionResponseMessage {
                        role: choice.message.role,
                        content: choice.message.content,
                        name: choice.message.name,
                        tool_call_id: choice.message.tool_call_id,
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
