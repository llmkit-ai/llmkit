import type { ApiLogReponse } from "./logs"

import type { Tool } from './tools'

export interface Prompt {
  id: number
  system: string
  user: string | null
  key: string
  model: string
  model_id: number
  provider: string
  max_tokens: number
  temperature: number
  json_mode: boolean
  json_schema: string | null
  prompt_type: string
  is_chat: boolean
  version_id: number
  version_number: number
  system_version_diff: string | null
  user_version_diff: string | null
  updated_at: string
  tools: Tool[]
  supports_json: boolean
  supports_json_schema: boolean
  supports_tools: boolean
  is_reasoning: boolean
  reasoning_effort: string | null
}


// Message type for chat
export interface Message {
  role: 'system' | 'user' | 'assistant';
  content: string;
  tool_calls?: ToolCall[];
  tool_call_id?: string; // For tool responses
  // Raw data handling - not sent to server but used for UI
  rawData?: string;
  showRawData?: boolean;
}

// Tool call structure
export interface ToolCall {
  // A unique identifier for the tool call, optional in streaming responses
  id?: string;
  // The index of the tool call in the list of tool calls
  index?: number;
  // The type of call. When streaming, the first chunk only will contain "function"
  type?: string;
  // The details of the function call
  function: FunctionCall;
}

export interface FunctionCall {
  // The name of the function to call, can be optional in streaming chunks
  name?: string;
  // A JSON string representing the arguments for the function call
  arguments: string;
}

// PROMPT EXECUTION RESPONSE
export interface PromptExecutionResponse {
  content: string;
  log: ApiLogReponse;
}

// OpenAI compatible API types
export interface ApiCompletionResponse {
  id: string;
  object: string;
  created: number;
  model: string;
  choices: ApiChoice[];
  usage: ApiUsage;
}

export interface ApiChoice {
  index: number;
  message: Message;
  finish_reason: string;
  tool_calls?: ToolCall[];
}

export interface ApiUsage {
  prompt_tokens: number;
  completion_tokens: number;
  total_tokens: number;
}

// For streaming responses
export interface ApiCompletionChunk {
  id: string;
  object: string;
  created: number;
  model: string;
  choices: ApiChunkChoice[];
}

export interface ApiChunkChoice {
  index: number;
  delta: ApiDelta;
  finish_reason?: string;
}

export interface ApiDelta {
  content?: string;
  role?: string;
  tool_calls?: ToolCall[];
}


// PROMPT PERFORMANCE
export interface PromptEvalVersionPerformanceResponse {
    version_id: number,
    version_number: number,
    version_date: string,
    avg_score: number | null,
    run_count: number,
}
