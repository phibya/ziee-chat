export interface GlobalConfig {
  id: string
  key: string
  value: unknown
  createdAt: string
  updatedAt: string
}

export interface GlobalConfigRequest {
  key: string
  value: unknown
}

export interface GlobalConfigResponse {
  configs: GlobalConfig[]
}

export interface DefaultLanguageResponse {
  language: string
}

export interface UpdateDefaultLanguageRequest {
  language: string
}

// Strongly typed global configuration keys and values
export interface GlobalConfigKeys {
  'appearance.defaultLanguage': 'en' | 'vi'
  // Future global configs can be added here
  // 'registration.enabled': boolean
  // 'maintenance.mode': boolean
}

// Helper type for setting a specific global config
export type SetGlobalConfigRequest<
  K extends keyof GlobalConfigKeys = keyof GlobalConfigKeys,
> = {
  key: K
  value: GlobalConfigKeys[K]
}

// Helper type for getting a specific global config
export type GetGlobalConfigResponse<
  K extends keyof GlobalConfigKeys = keyof GlobalConfigKeys,
> = {
  id: string
  key: K
  value: GlobalConfigKeys[K]
  createdAt: string
  updatedAt: string
}

// Default values for global configuration
export const DEFAULT_GLOBAL_CONFIG: GlobalConfigKeys = {
  'appearance.defaultLanguage': 'en',
}

// Type guard to check if a key is a valid global config key
export function isValidGlobalConfigKey(
  key: string,
): key is keyof GlobalConfigKeys {
  return key in DEFAULT_GLOBAL_CONFIG
}

// Helper function to get default value for a global config
export function getDefaultGlobalConfigValue<K extends keyof GlobalConfigKeys>(
  key: K,
): GlobalConfigKeys[K] {
  return DEFAULT_GLOBAL_CONFIG[key]
}
