use async_openai::types::{ChatCompletionMessageToolCall, ChatCompletionTool, ChatCompletionToolType, CreateChatCompletionResponse, CreateChatCompletionStreamResponse, FunctionCall, FunctionObject, ResponseFormat, ResponseFormatJsonSchema};

use crate::common::types::{chat_request::{ChatCompletionRequestJsonSchema, ChatCompletionRequestResponseFormat, ChatCompletionRequestTool, ChatCompletionRequestToolCall}, chat_response::{
    CompletionTokensDetails, LlmServiceChatCompletionChunk, LlmServiceChatCompletionResponse, LlmServiceChatCompletionResponseChoice, LlmServiceChatCompletionResponseFunctionCall, LlmServiceChatCompletionResponseMessage, LlmServiceChatCompletionResponseToolCall, LlmServiceChatCompletionResponseUsage, LlmServiceChoiceStream, LlmServiceStreamDelta, LlmServiceUsage, PromptTokensDetails
}};


// REQUEST MAPPINGS

impl From<ChatCompletionRequestTool> for ChatCompletionTool {
    fn from(value: ChatCompletionRequestTool) -> Self {
        match value {
            ChatCompletionRequestTool::Function { function } => {
                ChatCompletionTool {
                    r#type: async_openai::types::ChatCompletionToolType::Function,
                    function: FunctionObject {
                        name: function.name,
                        description: function.description,
                        parameters: Some(function.parameters),
                        strict: function.strict
                    },
                }
            }
        }
    }
}

impl From<ChatCompletionRequestResponseFormat> for ResponseFormat {
    fn from(value: ChatCompletionRequestResponseFormat) -> Self {
        match value.format_type.as_str() {
            "json_object" => ResponseFormat::JsonObject,
            "json_schema" => {
                match value.json_schema {
                    Some(js) => ResponseFormat::JsonSchema{
                        json_schema: js.into()
                    },
                    None => ResponseFormat::JsonObject
                }
            },
            _ => ResponseFormat::Text
        }
    }
}

impl From<ChatCompletionRequestJsonSchema> for ResponseFormatJsonSchema {
    fn from(value: ChatCompletionRequestJsonSchema) -> Self {
        ResponseFormatJsonSchema {
            name: value.name,
            strict: Some(value.strict),
            schema: Some(value.schema),
            description: None
        }
    }
}

impl From<ChatCompletionRequestToolCall> for ChatCompletionMessageToolCall {
    fn from(value: ChatCompletionRequestToolCall) -> Self {
        ChatCompletionMessageToolCall {
            id: value.id,
            r#type: ChatCompletionToolType::Function,
            function: FunctionCall {
                name: value.function_call.name.clone(),
                arguments: value.function_call.arguments.clone(),
            }
        }
    }
}


// RESPONSE MAPPING
impl From<CreateChatCompletionResponse> for LlmServiceChatCompletionResponse {
    fn from(value: CreateChatCompletionResponse) -> Self {
        LlmServiceChatCompletionResponse {
            id: value.id,
            choices: value
                .choices
                .into_iter()
                .enumerate()
                .map(|(index, choice)| LlmServiceChatCompletionResponseChoice {
                    index: index as u32,
                    message: LlmServiceChatCompletionResponseMessage {
                        role: choice.message.role.to_string(),
                        content: choice.message.content,
                        name: None,
                        tool_call_id: None,
                        tool_calls: choice.message.tool_calls.map(|tcs| {
                            tcs
                                .into_iter()
                                .map(|tc| LlmServiceChatCompletionResponseToolCall {
                                    id: Some(tc.id),
                                    index: None,
                                    kind: Some("function".to_string()),
                                    function_call: LlmServiceChatCompletionResponseFunctionCall {
                                        name: Some(tc.function.name),
                                        arguments: tc.function.arguments,
                                    },
                                })
                                .collect()
                        }),
                    },
                    finish_reason: choice.finish_reason.map(|fr| {
                        match fr {
                            async_openai::types::FinishReason::Stop => "stop".to_string(),
                            async_openai::types::FinishReason::Length => "length".to_string(),
                            async_openai::types::FinishReason::FunctionCall => "function_call".to_string(),
                            async_openai::types::FinishReason::ContentFilter => "content_filter".to_string(),
                            async_openai::types::FinishReason::ToolCalls => "tool_calls".to_string(),
                        }
                    }),
                    native_finish_reason: None,
                })
                .collect(),
            created: value.created as i64,
            model: value.model,
            object: "chat.completion".to_string(),
            usage: value
                .usage
                .map(|usage| LlmServiceChatCompletionResponseUsage {
                    prompt_tokens: usage.prompt_tokens,
                    completion_tokens: usage.completion_tokens,
                    total_tokens: usage.total_tokens,
                    prompt_tokens_details: usage.prompt_tokens_details.map(|details| PromptTokensDetails {
                        audio_tokens: details.audio_tokens,
                        cached_tokens: details.cached_tokens,
                    }),
                    completion_tokens_details: usage.completion_tokens_details.map(|details| CompletionTokensDetails {
                        accepted_prediction_tokens: details.accepted_prediction_tokens,
                        audio_tokens: details.audio_tokens,
                        reasoning_tokens: details.reasoning_tokens,
                        rejected_prediction_tokens: details.rejected_prediction_tokens,
                    }),
                }),
        }
    }
}


impl From<CreateChatCompletionStreamResponse> for LlmServiceChatCompletionChunk {
    fn from(chunk: CreateChatCompletionStreamResponse) -> Self {
        LlmServiceChatCompletionChunk {
            id: chunk.id,
            choices: chunk.choices.into_iter().map(|choice| {
                LlmServiceChoiceStream {
                    index: choice.index,
                    delta: LlmServiceStreamDelta {
                        role: choice.delta.role.map(|r| r.to_string()),
                        content: choice.delta.content,
                        tool_calls: choice.delta.tool_calls
                            .map(|tc|
                                tc.into_iter().map(|tool_call| {
                                    LlmServiceChatCompletionResponseToolCall {
                                        id: tool_call.id,
                                        index: Some(tool_call.index),
                                        kind: tool_call.r#type.map(|_| "function".to_string()),
                                        function_call: LlmServiceChatCompletionResponseFunctionCall {
                                            name: tool_call.function.as_ref().map(|f| f.name.clone()).flatten(),
                                            // TODO: Look into how to handle this properly
                                            arguments: tool_call.function.map(|f| f.arguments).flatten().unwrap_or("".to_string()),
                                        },
                                    }
                                }).collect()
                            )
                        
                    },
                    finish_reason: choice.finish_reason.map(|fr| {
                        match fr {
                            async_openai::types::FinishReason::Stop => "stop".to_string(),
                            async_openai::types::FinishReason::Length => "length".to_string(),
                            async_openai::types::FinishReason::FunctionCall => "function_call".to_string(),
                            async_openai::types::FinishReason::ContentFilter => "content_filter".to_string(),
                            async_openai::types::FinishReason::ToolCalls => "tool_calls".to_string(),
                        }
                    }),
                    native_finish_reason: None,
                }
            }).collect(),
            usage: chunk.usage.map(|usage| {
                LlmServiceUsage {
                    prompt_tokens: usage.prompt_tokens,
                    completion_tokens: usage.completion_tokens,
                    total_tokens: usage.total_tokens,
                    prompt_tokens_details: usage.prompt_tokens_details.map(|details| PromptTokensDetails {
                        audio_tokens: details.audio_tokens,
                        cached_tokens: details.cached_tokens,
                    }),
                    completion_tokens_details: usage.completion_tokens_details.map(|details| CompletionTokensDetails {
                        accepted_prediction_tokens: details.accepted_prediction_tokens,
                        audio_tokens: details.audio_tokens,
                        reasoning_tokens: details.reasoning_tokens,
                        rejected_prediction_tokens: details.rejected_prediction_tokens,
                    }),
                }
            }),
        }
    }
}
