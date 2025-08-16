/**
 * API Proxy Server type definitions
 */

// Configuration
export interface ApiProxyServerConfig {
  address: string
  port: number
  prefix: string
  api_key?: string
  allow_cors: boolean
  log_level: string
}

// Status
export interface ApiProxyServerStatus {
  running: boolean
  active_models?: number
  uptime?: number
}

// Model in proxy
export interface ApiProxyServerModel {
  id: string
  model_id: string
  alias_id?: string
  enabled: boolean
  is_default: boolean
  created_at: string
  updated_at: string
}

// Trusted host
export interface ApiProxyServerTrustedHost {
  id: string
  host: string
  description?: string
  enabled: boolean
  created_at: string
  updated_at: string
}

// Request types
export interface CreateApiProxyServerModelRequest {
  model_id: string
  alias_id?: string
  enabled?: boolean
  is_default?: boolean
}

export interface UpdateApiProxyServerModelRequest {
  alias_id?: string
  enabled?: boolean
  is_default?: boolean
}

export interface CreateTrustedHostRequest {
  host: string
  description?: string
  enabled?: boolean
}

export interface UpdateTrustedHostRequest {
  host?: string
  description?: string
  enabled?: boolean
}

// Response types
export interface ApiProxyServerModelListResponse {
  models: ApiProxyServerModel[]
  total: number
}

export interface ApiProxyServerTrustedHostListResponse {
  hosts: ApiProxyServerTrustedHost[]
  total: number
}
