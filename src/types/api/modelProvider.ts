/**
 * Model Provider API type definitions
 * Types for managing model providers in the application
 */

export interface ModelCapabilities {
  vision?: boolean
  audio?: boolean
  tools?: boolean
  codeInterpreter?: boolean
}

export interface ModelParameters {
  // Llama.cpp specific parameters
  contextSize?: number
  gpuLayers?: number
  temperature?: number
  topK?: number
  topP?: number
  minP?: number
  repeatLastN?: number
  repeatPenalty?: number
  presencePenalty?: number
  frequencyPenalty?: number
}

export interface ModelProviderModel {
  id: string
  name: string
  description?: string
  path?: string // For Llama.cpp models
  isDeprecated?: boolean
  isActive?: boolean
  enabled?: boolean
  capabilities?: ModelCapabilities
  parameters?: ModelParameters
}

export interface ModelProviderSettings {
  // Llama.cpp specific settings
  autoUnloadOldModels?: boolean
  contextShift?: boolean
  continuousBatching?: boolean
  parallelOperations?: number
  cpuThreads?: number
  threadsBatch?: number
  flashAttention?: boolean
  caching?: boolean
  kvCacheType?: string
  mmap?: boolean
  huggingFaceAccessToken?: string

  // Custom settings for other providers
  [key: string]: any
}

export interface ModelProvider {
  id: string
  name: string
  type: ModelProviderType
  icon?: string
  enabled: boolean
  api_key?: string
  base_url?: string
  models: ModelProviderModel[]
  settings?: ModelProviderSettings
  is_default?: boolean
  created_at?: string
  updated_at?: string
}

export type ModelProviderType =
  | 'llama.cpp'
  | 'openai'
  | 'anthropic'
  | 'groq'
  | 'gemini'
  | 'mistral'
  | 'custom'

export interface CreateModelProviderRequest {
  name: string
  type: ModelProviderType
  enabled?: boolean
  api_key?: string
  base_url?: string
  settings?: ModelProviderSettings
}

export interface UpdateModelProviderRequest {
  name?: string
  enabled?: boolean
  api_key?: string
  base_url?: string
  settings?: ModelProviderSettings
}

export interface ModelProviderListResponse {
  providers: ModelProvider[]
  total: number
  page: number
  per_page: number
}

export interface CloneModelProviderRequest {
  sourceId: string
  name: string
}

export interface AddModelToProviderRequest {
  name: string
  description?: string
  path?: string
  enabled?: boolean
  capabilities?: ModelCapabilities
  parameters?: ModelParameters
}

export interface UpdateModelRequest {
  name?: string
  description?: string
  path?: string
  enabled?: boolean
  isActive?: boolean
  capabilities?: ModelCapabilities
  parameters?: ModelParameters
}

export interface RemoveModelFromProviderRequest {
  providerId: string
  modelId: string
}
