import { create } from 'zustand'
import { ApiClient } from '../api/client'
import {
  DEFAULT_USER_SETTINGS,
  getDefaultUserSettingValue,
  isValidUserSettingKey,
  UserSetting,
  UserSettingKeys,
} from '../types'

interface UserSettingsState {
  // Current user settings
  settings: Partial<UserSettingKeys>

  // Global default language (fallback when user hasn't set language preference)
  globalDefaultLanguage: 'en' | 'vi'

  // Loading states
  loading: boolean
  initializing: boolean
}

export const useUserSettingsStore = create<UserSettingsState>(() => ({
  // Initial state
  settings: {},
  globalDefaultLanguage: 'en',
  loading: false,
  initializing: false,
}))

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
      globalDefaultLanguage: response.language as 'en' | 'vi',
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

// Computed getters
export const getUserAppearanceTheme = (): 'light' | 'dark' | 'system' => {
  return getUserSetting('appearance.theme')
}

export const getUserAppearanceComponentSize = (): 'small' | 'medium' | 'large' => {
  return getUserSetting('appearance.componentSize')
}

export const getUserAppearanceLanguage = (): 'en' | 'vi' => {
  const state = useUserSettingsStore.getState()
  // If user has explicitly set language preference, use it
  // Check if the key exists in settings (not just if it's truthy)
  if ('appearance.language' in state.settings) {
    return state.settings['appearance.language']!
  }
  // Otherwise, use global default language
  return state.globalDefaultLanguage
}

export const getUILeftPanelCollapsed = (): boolean => {
  return getUserSetting('ui.leftPanelCollapsed')
}

export const getUILeftPanelWidth = (): number => {
  return getUserSetting('ui.leftPanelWidth')
}

export const getResolvedAppearanceTheme = (): 'light' | 'dark' => {
  const theme = getUserAppearanceTheme()

  if (theme === 'system') {
    // Check system preference
    if (typeof window !== 'undefined' && window.matchMedia) {
      return window.matchMedia('(prefers-color-scheme: dark)').matches
        ? 'dark'
        : 'light'
    }
    return 'light'
  }

  return theme
}

// UI actions
export const setUILeftPanelCollapsed = async (
  collapsed: boolean,
): Promise<void> => {
  await saveUserSetting('ui.leftPanelCollapsed', collapsed)
}

export const setUILeftPanelWidth = async (width: number): Promise<void> => {
  await saveUserSetting('ui.leftPanelWidth', width)
}

// Helper hook for appearance settings
export const useAppearanceSettings = () => {
  const { loading } = useUserSettingsStore()

  return {
    theme: getUserAppearanceTheme(),
    componentSize: getUserAppearanceComponentSize(),
    language: getUserAppearanceLanguage(),
    resolvedTheme: getResolvedAppearanceTheme(),
    setTheme: (theme: 'light' | 'dark' | 'system') =>
      saveUserSetting('appearance.theme', theme),
    setComponentSize: (componentSize: 'small' | 'medium' | 'large') =>
      saveUserSetting('appearance.componentSize', componentSize),
    setLanguage: (language: 'en' | 'vi') =>
      saveUserSetting('appearance.language', language),
    loading,
  }
}

// Helper hook for UI settings
export const useUISettings = () => {
  const { loading } = useUserSettingsStore()

  return {
    leftPanelCollapsed: getUILeftPanelCollapsed(),
    leftPanelWidth: getUILeftPanelWidth(),
    setLeftPanelCollapsed: setUILeftPanelCollapsed,
    setLeftPanelWidth: setUILeftPanelWidth,
    loading,
  }
}

// Initialize settings on app start
export const initializeUserSettingsOnStartup = async (): Promise<void> => {
  // Load both user settings and global language configuration
  await Promise.all([loadUserSettings(), loadGlobalDefaultLanguage()])
}
