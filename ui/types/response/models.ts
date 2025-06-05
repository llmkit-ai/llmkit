export interface Model {
  id: number,
  provider_id: number,
  name: string,
  provider_name: string,
  provider_base_url: string,
  supports_json: boolean,
  supports_json_schema: boolean,
  supports_tools: boolean,
  is_reasoning: boolean,
}
