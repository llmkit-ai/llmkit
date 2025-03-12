export interface PromptCreateDTO {
  key: string;
  system: string;
  user?: string | null;
  model_id: number | null;
  max_tokens: number;
  temperature: number;
  json_mode: boolean;
  json_schema: string | null;
  prompt_type: string;
  is_chat: boolean;
  tool_version_ids?: number[];
}

export interface PromptUpdateDTO {
  id: number;
  key: string;
  system: string;
  user?: string | null;
  model_id: number | null;
  max_tokens: number;
  temperature: number;
  json_mode: boolean;
  json_schema: string | null;
  prompt_type: string;
  is_chat: boolean;
  tool_version_ids?: number[];
}

