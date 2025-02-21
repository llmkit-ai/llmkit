
// EXECUTION RESPONSE
export interface PromptEvalExecutionRunResponse {
  run_id: string;
  runs: PromptEvalRunResponse[];
}


// GET EVAL RESPONSE
export interface PromptEvalRunResponse {
  id: number;
  run_id: string;
  prompt_version_id: number;
  prompt_eval_id: number;
  prompt_eval_name: string;
  score: number | null;
  output: string;
  created_at: string;
  updated_at: string;
}
