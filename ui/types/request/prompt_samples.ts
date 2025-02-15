export interface CreatePromptSampleRequest {
  prompt_id: number
  input_data: Object
  name: string
}

export interface UpdatePromptSampleRequest {
  input_data?: Object
  name?: string
}
