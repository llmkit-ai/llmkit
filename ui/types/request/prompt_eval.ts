export interface CreatePromptEvalRequest {
  prompt_id: number
  input_data: Object
  name: string
}

export interface UpdatePromptEvalRequest {
  input_data?: Object
  name?: string
}
