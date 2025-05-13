use serde::{Deserialize, Serialize};


/// Chat completion request matching the OpenAi API schema.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatCompletionRequest {
    /// The model ID to use.
    pub model: String,
    /// The list of messages.
    pub messages: Vec<ChatCompletionRequestMessage>,
    /// Whether the response should be streamed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    /// (Optional) Stub for response_format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ChatCompletionRequestResponseFormat>,
    /// (Optional) Tool calling field. Now uses our production‑ready tool types.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ChatCompletionRequestTool>>,
    /// (Optional) Stub for provider preferences.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// (Optional) Fallback models.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub models: Option<Vec<String>>,
    /// (Optional) Message transforms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transforms: Option<Vec<String>>,
    /// Maximum number of tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i64>,
    /// What sampling temperature to use, between 0 and 2
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct ChatCompletionRequestResponseFormat {
    /// Currently only supports "json_object" as per OpenAI spec
    #[serde(rename = "type")]
    pub format_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json_schema: Option<ChatCompletionRequestJsonSchema>
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct ChatCompletionRequestJsonSchema {
    pub name: String,
    pub strict: bool,
    pub schema: serde_json::Value
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase", tag = "role")]
pub enum ChatCompletionRequestMessage{
    System { 
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
    User { 
        content: String ,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Assistant { 
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_calls: Option<Vec<ChatCompletionRequestToolCall>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatCompletionRequestFunctionCall {
    /// The name of the function to call.
    pub name: String,
    /// A JSON string representing the arguments for the function call.
    pub arguments: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatCompletionRequestToolCall {
    /// A unique identifier for the tool call.
    pub id: String,
    /// The type of call. It must be "function" for function calls.
    #[serde(rename = "type")]
    pub kind: String,
    /// The details of the function call, including its function name and arguments.
    #[serde(rename = "function")]
    pub function_call: ChatCompletionRequestFunctionCall,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ChatCompletionRequestTool {
    /// A function call tool with an associated [FunctionDescription].
    Function {
        #[serde(rename = "function")]
        function: ChatCompletionRequestFunctionDescription,
    },
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatCompletionRequestFunctionDescription {
    /// The name of the function.
    pub name: String,
    /// An optional description of what the function does.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// A JSON Schema object representing the function parameters.
    /// This should be a valid JSON object describing the expected arguments.
    pub parameters: serde_json::Value,
}

// Helper Methods for easy extraction
impl ChatCompletionRequestMessage {
    /// Returns the content of the message regardless of its role
    pub fn content(&self) -> &str {
        match self {
            ChatCompletionRequestMessage::System { content, .. } => content,
            ChatCompletionRequestMessage::User { content, .. } => content,
            ChatCompletionRequestMessage::Assistant { content, .. } => content,
        }
    }

    /// Returns the name of the message if available
    pub fn name(&self) -> Option<&str> {
        match self {
            ChatCompletionRequestMessage::System { name, .. } => name.as_deref(),
            ChatCompletionRequestMessage::User { name, .. } => name.as_deref(),
            ChatCompletionRequestMessage::Assistant { name, .. } => name.as_deref(),
        }
    }

    /// Returns the role of the message as a string
    pub fn role(&self) -> &'static str {
        match self {
            ChatCompletionRequestMessage::System { .. } => "system",
            ChatCompletionRequestMessage::User { .. } => "user",
            ChatCompletionRequestMessage::Assistant { .. } => "assistant",
        }
    }

    /// Returns the tool calls if this message is from an assistant
    pub fn tool_calls(&self) -> Option<&Vec<ChatCompletionRequestToolCall>> {
        match self {
            ChatCompletionRequestMessage::Assistant { tool_calls, .. } => tool_calls.as_ref(),
            _ => None,
        }
    }

    /// Check if this message is from the system
    pub fn is_system(&self) -> bool {
        matches!(self, ChatCompletionRequestMessage::System { .. })
    }

    /// Check if this message is from a user
    pub fn is_user(&self) -> bool {
        matches!(self, ChatCompletionRequestMessage::User { .. })
    }

    /// Check if this message is from an assistant
    pub fn is_assistant(&self) -> bool {
        matches!(self, ChatCompletionRequestMessage::Assistant { .. })
    }
}

