use crate::common::types::chat_request::{
    ChatCompletionRequestFunctionCall, ChatCompletionRequestFunctionDescription,
    ChatCompletionRequestJsonSchema, ChatCompletionRequestResponseFormat,
    ChatCompletionRequestTool, ChatCompletionRequestToolCall,
};
use openrouter_api::models::tool::{FunctionCall, ToolCall};

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
            id: self.id,
            kind: self.kind,
            function_call: self.function_call.into(),
        }
    }
}

// Impl for Opernrouter SDK
impl Into<FunctionCall> for ChatCompletionRequestFunctionCall {
    fn into(self) -> FunctionCall {
        FunctionCall {
            name: self.name,
            arguments: self.arguments,
        }
    }
}
