/**
 * Assistant API types - matching backend structure
 */

export interface Assistant {
  id: string
  name: string
  description?: string
  instructions?: string
  parameters?: Record<string, any>
  created_by?: string
  is_template: boolean
  is_active: boolean
  created_at: string
  updated_at: string
}

export interface CreateAssistantRequest {
  name: string
  description?: string
  instructions?: string
  parameters?: Record<string, any>
  is_template?: boolean
}

export interface UpdateAssistantRequest {
  name?: string
  description?: string
  instructions?: string
  parameters?: Record<string, any>
  is_template?: boolean
  is_active?: boolean
}

export interface AssistantListResponse {
  assistants: Assistant[]
  total: number
  page: number
  per_page: number
}

// Default assistant parameters
export const DEFAULT_ASSISTANT_PARAMETERS = {
  stream: true,
  temperature: 0.7,
  frequency_penalty: 0.7,
  presence_penalty: 0.7,
  top_p: 0.95,
  top_k: 2,
} as const
