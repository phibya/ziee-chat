// Configuration types
export interface UserRegistrationStatusResponse {
  enabled: boolean
}

export interface UpdateUserRegistrationRequest {
  enabled: boolean
}

export interface ProxySettingsResponse {
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

export interface UpdateProxySettingsRequest {
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

export interface TestProxyConnectionRequest {
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

export interface TestProxyConnectionResponse {
  success: boolean
  message: string
}

// Ngrok configuration types
export interface NgrokSettingsResponse {
  api_key: string
  tunnel_enabled: boolean
  tunnel_url: string | null
  tunnel_status: string
  auto_start: boolean
}

export interface UpdateNgrokSettingsRequest {
  api_key?: string
  tunnel_enabled?: boolean
  auto_start?: boolean
}

export interface UpdateAccountPasswordRequest {
  current_password?: string  // Optional for desktop apps
  new_password: string
}

export interface NgrokStatusResponse {
  tunnel_active: boolean
  tunnel_url: string | null
  tunnel_status: string
  last_error: string | null
}
