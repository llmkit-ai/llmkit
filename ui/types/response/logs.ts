export interface ApiLogReponse {
  id: number;
  prompt_id: number | null;
  model_id: number;
  model_name: string;
  status_code: number | null;
  latency_ms: number | null;
  input_tokens: number | null;
  output_tokens: number | null;
  reasoning_tokens: number | null;
  request_body: string | null;
  response_data: string | null;
  provider_response_id: string;
  created_at: string
}

export type ApiLogResponseArray = ApiLogReponse[];
