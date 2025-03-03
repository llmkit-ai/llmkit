export interface PromptEvalResponse {
  id: number
  prompt_id: number
  system_prompt_input?: string
  user_prompt_input: string
  name: string
  created_at: string
  updated_at: string
}
