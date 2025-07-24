import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { useShallow } from 'zustand/react/shallow'
import { ApiClient } from '../api/client'
import {
  DEFAULT_USER_SETTINGS,
  getDefaultUserSettingValue,
  isValidUserSettingKey,
  SupportedLanguage,
  UserSetting,
  UserSettingKeys,
} from '../types'

interface UserSettingsState {
  // Current user settings
  settings: Partial<UserSettingKeys>

  // Global default language (fallback when user hasn't set language preference)
  globalDefaultLanguage: SupportedLanguage

  // Loading states
  loading: boolean
  initializing: boolean
}

export const useUserSettingsStore = create(
  subscribeWithSelector(
    (): UserSettingsState => ({
      // Initial state
      settings: {},
      globalDefaultLanguage: 'en',
      loading: false,
      initializing: false,
    }),
  ),
)

// Settings actions
export const loadUserSettings = async (): Promise<void> => {
  useUserSettingsStore.setState({ initializing: true })

  try {
    const response = await ApiClient.UserSettings.getAll()
    const settingsMap: Partial<UserSettingKeys> = {}

    response.settings.forEach((setting: UserSetting) => {
      if (isValidUserSettingKey(setting.key)) {
        // Type assertion is safe here because we validated the key
        settingsMap[setting.key as keyof UserSettingKeys] = setting.value as any
      }
    })

    useUserSettingsStore.setState({ settings: settingsMap })
  } catch (error) {
    console.error('Failed to load user settings:', error)
    // Use default settings if loading fails
    useUserSettingsStore.setState({ settings: DEFAULT_USER_SETTINGS })
  } finally {
    useUserSettingsStore.setState({ initializing: false })
  }
}

export const loadGlobalDefaultLanguage = async (): Promise<void> => {
  try {
    const response = await ApiClient.Config.getDefaultLanguage()
    useUserSettingsStore.setState({
      globalDefaultLanguage: response.language as SupportedLanguage,
    })
  } catch (error) {
    console.error('Failed to load global default language:', error)
    // Keep default 'en' if loading fails
  }
}

export const getUserSetting = <K extends keyof UserSettingKeys>(
  key: K,
): UserSettingKeys[K] => {
  const state = useUserSettingsStore.getState()
  return state.settings[key] ?? getDefaultUserSettingValue(key)
}

export const saveUserSetting = async <K extends keyof UserSettingKeys>(
  key: K,
  value: UserSettingKeys[K],
): Promise<void> => {
  useUserSettingsStore.setState({ loading: true })

  try {
    await ApiClient.UserSettings.set({
      key,
      value,
    })

    // Update local state
    useUserSettingsStore.setState(state => ({
      settings: {
        ...state.settings,
        [key]: value,
      },
    }))
  } catch (error) {
    console.error(`Failed to set setting ${key}:`, error)
    throw error
  } finally {
    useUserSettingsStore.setState({ loading: false })
  }
}

export const updateUserSetting = <K extends keyof UserSettingKeys>(
  key: K,
  value: UserSettingKeys[K],
): void => {
  useUserSettingsStore.setState(state => ({
    settings: {
      ...state.settings,
      [key]: value,
    },
  }))
}

export const deleteUserSetting = async (
  key: keyof UserSettingKeys,
): Promise<void> => {
  useUserSettingsStore.setState({ loading: true })

  try {
    await ApiClient.UserSettings.delete({ key })

    // Remove from local state
    useUserSettingsStore.setState(state => {
      const newSettings = { ...state.settings }
      delete newSettings[key]
      return { settings: newSettings }
    })
  } catch (error) {
    console.error(`Failed to delete setting ${key}:`, error)
    throw error
  } finally {
    useUserSettingsStore.setState({ loading: false })
  }
}

export const resetAllUserSettings = async (): Promise<void> => {
  useUserSettingsStore.setState({ loading: true })

  try {
    await ApiClient.UserSettings.deleteAll()
    useUserSettingsStore.setState({ settings: {} })
  } catch (error) {
    console.error('Failed to reset settings:', error)
    throw error
  } finally {
    useUserSettingsStore.setState({ loading: false })
  }
}

export const useUserSettings = <K extends keyof UserSettingKeys>(
  key: K,
): UserSettingKeys[K] => {
  return (
    useUserSettingsStore(useShallow(state => state.settings[key])) ??
    getDefaultUserSettingValue(key)
  )
}

export const useUserAppearanceLanguage = (): SupportedLanguage => {
  const globalDefaultLanguage = useUserSettingsStore(
    useShallow(state => state.globalDefaultLanguage),
  )
  return (
    useUserSettingsStore(
      useShallow(state => state.settings['appearance.language']),
    ) ??
    globalDefaultLanguage ??
    getDefaultUserSettingValue('appearance.language')
  )
}

export const setUserAppearanceLanguage = async (
  language: SupportedLanguage,
): Promise<void> => {
  await saveUserSetting('appearance.language', language)
}

export const useUILeftPanelCollapsed = (): boolean => {
  return useUserSettings('ui.leftPanelCollapsed')
}

export const setUILeftPanelCollapsed = async (
  collapsed: boolean,
): Promise<void> => {
  await saveUserSetting('ui.leftPanelCollapsed', collapsed)
}

export const useUILeftPanelWidth = (): number => {
  return useUserSettings('ui.leftPanelWidth')
}

export const setUILeftPanelWidth = async (width: number): Promise<void> => {
  await saveUserSetting('ui.leftPanelWidth', width)
}

export const useUserAppearanceTheme = (): 'light' | 'dark' | 'system' => {
  return useUserSettings('appearance.theme')
}

export const setUserAppearanceTheme = async (
  theme: 'light' | 'dark' | 'system',
): Promise<void> => {
  await saveUserSetting('appearance.theme', theme)
}

// Initialize settings on app start
export const initializeUserSettings = async (): Promise<void> => {
  // Load both user settings and global language configuration
  await Promise.all([loadUserSettings(), loadGlobalDefaultLanguage()])
}
