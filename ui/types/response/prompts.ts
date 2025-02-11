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
}


// PROMPT EXECUTION RESPONSE
export interface PromptExecutionResponse {
  content: string;
  log: ApiLogReponse;
}
