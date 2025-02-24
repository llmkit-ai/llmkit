export interface PromptCreateDTO {
  key: string;
  system: string;
  user: string;
  model_id: number | null;
  max_tokens: number;
  temperature: number;
  json_mode: boolean;
  prompt_type: string;
  is_chat: boolean;
}

export interface PromptUpdateDTO {
  id: number;
  key: string;
  system: string;
  user: string;
  model_id: number | null;
  max_tokens: number;
  temperature: number;
  json_mode: boolean;
  prompt_type: string;
  is_chat: boolean;
}

