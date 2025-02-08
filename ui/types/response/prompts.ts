export interface Prompt {
  id: number
  prompt: string
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
  log: PromptExecutionApiTraceResponse;
}


export interface PromptExecutionApiTraceResponse {
  id: number;
  prompt_id: number | null;
  model_id: number;
  response_data: string | null;
  status_code: number | null;
  latency_ms: number | null;
  input_tokens: number | null;
  output_tokens: number | null;
  reasoning_tokens: number | null;
  request_body: string | null;
}
