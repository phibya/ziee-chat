/**
 * Provider API type definitions
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

export interface ModelSettings {
  /// Set verbose mode (print all requests)
  verbose?: boolean
  /// Maximum number of sequences to allow (default: 256)
  max_num_seqs?: number
  /// Size of a block (default: 32)
  block_size?: number
  /// Available GPU memory for kvcache in MB (default: 4096)
  kvcache_mem_gpu?: number
  /// Available CPU memory for kvcache in MB (default: 128)
  kvcache_mem_cpu?: number
  /// Record conversation (default: false, the client needs to record chat history)
  record_conversation?: boolean
  /// Maximum waiting time for processing parallel requests in milliseconds (default: 500)
  holding_time?: number
  /// Whether the program runs in multiprocess or multithread mode for parallel inference (default: false)
  multi_process?: boolean
  /// Enable logging (default: false)
  log?: boolean
  /// Model architecture (llama, mistral, etc.)
  architecture?: string
  /// Device type (cpu, cuda, metal, etc.)
  device_type?: string
  /// Array of device IDs for multi-GPU
  device_ids?: number[]
}

export interface Model {
  id: string
  name: string
  alias: string
  description?: string
  is_deprecated?: boolean
  is_active?: boolean
  enabled?: boolean
  capabilities?: ModelCapabilities
  parameters?: ModelParameters
  settings?: ModelSettings // Model-specific performance settings
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
}

export interface UpdateProviderRequest {
  name?: string
  enabled?: boolean
  api_key?: string
  base_url?: string
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
  settings?: ModelSettings
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
  settings?: ModelSettings
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
  id: number
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
