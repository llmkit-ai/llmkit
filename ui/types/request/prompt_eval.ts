export interface CreatePromptEvalRequest {
  prompt_id: number
  system_prompt_input?: Object
  user_prompt_input: Object | string
  name: string
}

export interface UpdatePromptEvalRequest {
  system_prompt_input?: Object
  user_prompt_input?: Object | string
  name?: string
}
