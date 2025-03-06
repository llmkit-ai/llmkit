export interface Tool {
  id: number
  name: string
  current_tool_version_id: number | null
  version_number: number
  tool_name: string
  description: string
  parameters: string
  strict: boolean
  version_id: number
  created_at: string
  updated_at: string
}

export interface ToolVersion {
  id: number
  tool_id: number
  version_number: number
  tool_name: string
  description: string
  parameters: string
  strict: boolean
  created_at: string
}