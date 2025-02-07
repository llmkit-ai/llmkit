export interface ApiTraceResponse {
  id: number;
  prompt_id: number | null;
  model_id: number;
  response_data: string | null;
  status_code: number | null;
  latency_ms: number | null;
  input_tokens: number | null;
  output_tokens: number | null;
  request_body: string | null;
  request_method: string | null;
  request_url: string | null;
  request_headers: string | null;
}

export type ApiTraceResponseArray = ApiTraceResponse[];
