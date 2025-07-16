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
  // Candle specific parameters
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

export interface Model {
  id: string
  name: string
  alias: string
  description?: string
  path?: string // For Candle models
  is_deprecated?: boolean
  is_active?: boolean
  enabled?: boolean
  capabilities?: ModelCapabilities
  parameters?: ModelParameters
  device_type?: string // cpu, cuda, metal, etc.
  device_ids?: string[] // Array of device IDs for multi-GPU
}

export interface ProviderSettings {
  // Candle specific settings
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

export interface ProviderProxySettings {
  enabled: boolean
  url: string
  username: string
  password: string
  no_proxy: string
  ignore_ssl_certificates: boolean
  proxy_ssl: boolean
  proxy_host_ssl: boolean
  peer_ssl: boolean
  host_ssl: boolean
}

export interface Provider {
  id: string
  name: string
  type: ProviderType
  icon?: string
  enabled: boolean
  api_key?: string
  base_url?: string
  settings?: ProviderSettings
  proxy_settings?: ProviderProxySettings
  is_default?: boolean
  created_at?: string
  updated_at?: string
}

export type ProviderType =
  | 'candle'
  | 'openai'
  | 'anthropic'
  | 'groq'
  | 'gemini'
  | 'mistral'
  | 'custom'

export interface CreateProviderRequest {
  name: string
  type: ProviderType
  enabled?: boolean
  api_key?: string
  base_url?: string
  settings?: ProviderSettings
  proxy_settings?: ProviderProxySettings
}

export interface UpdateProviderRequest {
  name?: string
  enabled?: boolean
  api_key?: string
  base_url?: string
  settings?: ProviderSettings
  proxy_settings?: ProviderProxySettings
}

export interface ProviderListResponse {
  providers: Provider[]
  total: number
  page: number
  per_page: number
}

export interface CloneProviderRequest {
  sourceId: string
  name: string
}

export interface AddModelToProviderRequest {
  name: string
  alias: string
  description?: string
  path?: string
  enabled?: boolean
  capabilities?: ModelCapabilities
  parameters?: ModelParameters
  device_type?: string
  device_ids?: string[]
}

export interface UpdateModelRequest {
  name?: string
  alias?: string
  description?: string
  path?: string
  enabled?: boolean
  is_active?: boolean
  capabilities?: ModelCapabilities
  parameters?: ModelParameters
  device_type?: string
  device_ids?: string[]
}

export interface RemoveModelFromProviderRequest {
  providerId: string
  modelId: string
}

// TestProviderProxyRequest removed - now using ProviderProxySettings directly

export interface TestProviderProxyResponse {
  success: boolean
  message: string
}

// Device detection types
export interface DeviceInfo {
  id: string // Device identifier - UUID for CUDA GPUs, or descriptive ID for other devices
  name: string
  device_type: string // cpu, cuda, metal
  memory_total?: number // Total memory in bytes
  memory_free?: number // Free memory in bytes
  is_available: boolean
}

export interface AvailableDevicesResponse {
  devices: DeviceInfo[]
  default_device_type: string
  supports_multi_gpu: boolean
}
