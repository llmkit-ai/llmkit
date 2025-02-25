import type { ApiLogReponse } from "./logs"

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
  prompt_type: string
  is_chat: boolean
  version_id: number
  version_number: number
  system_version_diff: string | null
  user_version_diff: string | null
  updated_at: string
}


// Message type for chat
export interface Message {
  role: 'system' | 'user' | 'assistant';
  content: string;
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
}


// PROMPT PERFORMANCE
export interface PromptEvalVersionPerformanceResponse {
    version_id: number,
    version_number: number,
    version_date: string,
    avg_score: number | null,
    run_count: number,
}
