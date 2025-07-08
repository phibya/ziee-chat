export interface UserSetting {
  id: string
  userId: string
  key: string
  value: unknown
  createdAt: string
  updatedAt: string
}

export interface UserSettingRequest {
  key: string
  value: unknown
}

export interface UserSettingsResponse {
  settings: UserSetting[]
}

// Strongly typed appearance settings
export interface AppearanceSettings {
  theme: 'light' | 'dark' | 'system'
  fontSize: number
}

// Strongly typed user setting keys and values
export interface UserSettingKeys {
  'appearance.theme': 'light' | 'dark' | 'system'
  'appearance.fontSize': number
  // Future settings can be added here
  // 'shortcuts.save': string
  // 'proxy.host': string
  // 'proxy.port': number
}

// Helper type for setting a specific user setting
export type SetUserSettingRequest<
  K extends keyof UserSettingKeys = keyof UserSettingKeys,
> = {
  key: K
  value: UserSettingKeys[K]
}

// Helper type for getting a specific user setting
export type GetUserSettingResponse<
  K extends keyof UserSettingKeys = keyof UserSettingKeys,
> = {
  id: string
  userId: string
  key: K
  value: UserSettingKeys[K]
  createdAt: string
  updatedAt: string
}

// Default values for user settings
export const DEFAULT_USER_SETTINGS: UserSettingKeys = {
  'appearance.theme': 'system',
  'appearance.fontSize': 14,
}

// Type guard to check if a key is a valid user setting key
export function isValidUserSettingKey(
  key: string,
): key is keyof UserSettingKeys {
  return key in DEFAULT_USER_SETTINGS
}

// Helper function to get default value for a setting
export function getDefaultUserSettingValue<K extends keyof UserSettingKeys>(
  key: K,
): UserSettingKeys[K] {
  return DEFAULT_USER_SETTINGS[key]
}
