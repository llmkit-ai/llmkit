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
    pub max_tokens: Option<u32>,
    /// What sampling temperature to use, between 0 and 2
    pub temperature: Option<f32>,
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
    Assistant { 
        content: Option<String>,
        tool_calls: Option<Vec<ChatCompletionRequestToolCall>>,
        name: Option<String>,
    },
    Tool { 
        content: String,
        tool_call_id: String
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

    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>
}

// Helper Methods for easy extraction
impl ChatCompletionRequestMessage {
    /// Returns the content of the message regardless of its role
    pub fn content(&self) -> Option<String> {
        match self {
            ChatCompletionRequestMessage::System { content, .. } => Some(content.clone()),
            ChatCompletionRequestMessage::User { content, .. } => Some(content.clone()),
            ChatCompletionRequestMessage::Assistant { content, .. } => content.clone().map(|c| c),
            ChatCompletionRequestMessage::Tool { content, .. } => Some(content.clone()),
        }
    }

    pub fn system_content(&self) -> String {
        match self {
            ChatCompletionRequestMessage::System { content, .. } => content.clone(),
            _ => "".to_string()
        }
    }

    pub fn user_content(&self) -> String {
        match self {
            ChatCompletionRequestMessage::User { content, .. } => content.clone(),
            _ => "".to_string()
        }
    }

    pub fn assistant_content(&self) -> Option<String> {
        match self {
            ChatCompletionRequestMessage::Assistant { content, .. } => content.clone(),
            _ => None
        }
    }

    pub fn tool_content(&self) -> String {
        match self {
            ChatCompletionRequestMessage::Tool { content, .. } => content.clone(),
            _ => "".to_string()
        }
    }

    pub fn tool_call_id(&self) -> Option<String> {
        match self {
            ChatCompletionRequestMessage::Tool { tool_call_id, .. } => Some(tool_call_id.clone()),
            _ => None
        }
    }

    /// Returns the name of the message if available
    pub fn name(&self) -> Option<&str> {
        match self {
            ChatCompletionRequestMessage::System { name, .. } => name.as_deref(),
            ChatCompletionRequestMessage::User { name, .. } => name.as_deref(),
            ChatCompletionRequestMessage::Assistant { name, .. } => name.as_deref(),
            _ => None
        }
    }

    /// Returns the role of the message as a string
    pub fn role(&self) -> &'static str {
        match self {
            ChatCompletionRequestMessage::System { .. } => "system",
            ChatCompletionRequestMessage::User { .. } => "user",
            ChatCompletionRequestMessage::Assistant { .. } => "assistant",
            ChatCompletionRequestMessage::Tool { .. } => "tool",
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

