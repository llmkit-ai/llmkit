import type { ApiLogReponse } from "./logs"

import type { ToolVersion } from './tools'

export interface Prompt {
  id: number
  system: string
  user: string
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
  tools: ToolVersion[]
}


// Message type for chat
export interface Message {
  role: 'system' | 'user' | 'assistant';
  content: string;
  tool_calls?: ToolCall[];
  tool_call_id?: string; // For tool responses
}

// Tool call structure
export interface ToolCall {
  id: string;
  type: 'function';
  function: {
    name: string;
    arguments: string;
  };
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
