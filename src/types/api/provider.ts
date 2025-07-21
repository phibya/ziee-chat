/**
 * Provider API type definitions
 * Types for managing model providers in the application
 */

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
  | 'local'
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
