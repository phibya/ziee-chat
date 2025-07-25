/**
 * Hub API type definitions
 */

export interface HubModel {
  id: string
  name: string
  alias: string
  description?: string
  repository_url: string
  repository_path: string
  main_filename: string
  file_format: string
  capabilities?: {
    vision?: boolean
    audio?: boolean
    tools?: boolean
    code_interpreter?: boolean
  }
  size_gb: number
  tags: string[]
  recommended_parameters?: any
  public: boolean
  popularity_score?: number
  license?: string
  quantization_options?: string[]
  context_length?: number
  language_support?: string[]
}

export interface HubAssistant {
  id: string
  name: string
  description?: string
  instructions?: string
  parameters?: any
  category: string
  tags: string[]
  recommended_models: string[]
  capabilities_required: string[]
  popularity_score?: number
  author?: string
  use_cases?: string[]
  example_prompts?: string[]
}

export interface HubData {
  models: HubModel[]
  assistants: HubAssistant[]
  hub_version: string
  last_updated: string
}

// Response types for specific endpoints
export interface HubDataResponse extends HubData {}

export interface HubVersionResponse {
  hub_version: string
}

// File structure types used by hub manager
export interface HubModelsFile {
  hub_version: string
  schema_version: number
  models: HubModel[]
}

export interface HubAssistantsFile {
  hub_version: string
  schema_version: number
  assistants: HubAssistant[]
}