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

  // Actions
  loadSettings: () => Promise<void>
  loadGlobalLanguage: () => Promise<void>
  getSetting: <K extends keyof UserSettingKeys>(key: K) => UserSettingKeys[K]
  setSetting: <K extends keyof UserSettingKeys>(
    key: K,
    value: UserSettingKeys[K],
  ) => Promise<void>
  updateSetting: <K extends keyof UserSettingKeys>(
    key: K,
    value: UserSettingKeys[K],
  ) => void
  deleteSetting: (key: keyof UserSettingKeys) => Promise<void>
  resetSettings: () => Promise<void>

  // Computed values
  getAppearanceTheme: () => 'light' | 'dark' | 'system'
  getAppearanceComponentSize: () => 'small' | 'medium' | 'large'
  getAppearanceLanguage: () => 'en' | 'vi'
  getLeftPanelCollapsed: () => boolean
  getLeftPanelWidth: () => number

  // Helper to get resolved theme (system -> light/dark)
  getResolvedTheme: () => 'light' | 'dark'

  // UI actions
  setLeftPanelCollapsed: (collapsed: boolean) => Promise<void>
  setLeftPanelWidth: (width: number) => Promise<void>
}

export const useUserSettingsStore = create<UserSettingsState>((set, get) => ({
  settings: {},
  globalDefaultLanguage: 'en',
  loading: false,
  initializing: false,

  loadSettings: async () => {
    set({ initializing: true })

    try {
      const response = await ApiClient.UserSettings.getAll()
      const settingsMap: Partial<UserSettingKeys> = {}

      response.settings.forEach((setting: UserSetting) => {
        if (isValidUserSettingKey(setting.key)) {
          // Type assertion is safe here because we validated the key
          settingsMap[setting.key as keyof UserSettingKeys] =
            setting.value as any
        }
      })

      set({ settings: settingsMap })
    } catch (error) {
      console.error('Failed to load user settings:', error)
      // Use default settings if loading fails
      set({ settings: DEFAULT_USER_SETTINGS })
    } finally {
      set({ initializing: false })
    }
  },

  loadGlobalLanguage: async () => {
    try {
      const response = await ApiClient.Config.getDefaultLanguage()
      set({ globalDefaultLanguage: response.language as 'en' | 'vi' })
    } catch (error) {
      console.error('Failed to load global default language:', error)
      // Keep default 'en' if loading fails
    }
  },

  getSetting: <K extends keyof UserSettingKeys>(key: K): UserSettingKeys[K] => {
    const state = get()
    return state.settings[key] ?? getDefaultUserSettingValue(key)
  },

  setSetting: async <K extends keyof UserSettingKeys>(
    key: K,
    value: UserSettingKeys[K],
  ) => {
    set({ loading: true })

    try {
      await ApiClient.UserSettings.set({
        key,
        value,
      })

      // Update local state
      set(state => ({
        settings: {
          ...state.settings,
          [key]: value,
        },
      }))
    } catch (error) {
      console.error(`Failed to set setting ${key}:`, error)
      throw error
    } finally {
      set({ loading: false })
    }
  },

  updateSetting: <K extends keyof UserSettingKeys>(
    key: K,
    value: UserSettingKeys[K],
  ) => {
    set(state => ({
      settings: {
        ...state.settings,
        [key]: value,
      },
    }))
  },

  deleteSetting: async (key: keyof UserSettingKeys) => {
    set({ loading: true })

    try {
      await ApiClient.UserSettings.delete({ key })

      // Remove from local state
      set(state => {
        const newSettings = { ...state.settings }
        delete newSettings[key]
        return { settings: newSettings }
      })
    } catch (error) {
      console.error(`Failed to delete setting ${key}:`, error)
      throw error
    } finally {
      set({ loading: false })
    }
  },

  resetSettings: async () => {
    set({ loading: true })

    try {
      await ApiClient.UserSettings.deleteAll()
      set({ settings: {} })
    } catch (error) {
      console.error('Failed to reset settings:', error)
      throw error
    } finally {
      set({ loading: false })
    }
  },

  getAppearanceTheme: () => {
    const state = get()
    return state.getSetting('appearance.theme')
  },

  getAppearanceComponentSize: () => {
    const state = get()
    return state.getSetting('appearance.componentSize')
  },

  getAppearanceLanguage: () => {
    const state = get()
    // If user has explicitly set language preference, use it
    if (state.settings['appearance.language']) {
      return state.settings['appearance.language']
    }
    // Otherwise, use global default language
    return state.globalDefaultLanguage
  },

  getLeftPanelCollapsed: () => {
    const state = get()
    return state.getSetting('ui.leftPanelCollapsed')
  },

  getLeftPanelWidth: () => {
    const state = get()
    return state.getSetting('ui.leftPanelWidth')
  },

  getResolvedTheme: () => {
    const state = get()
    const theme = state.getAppearanceTheme()

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
  },

  setLeftPanelCollapsed: async (collapsed: boolean) => {
    const state = get()
    await state.setSetting('ui.leftPanelCollapsed', collapsed)
  },

  setLeftPanelWidth: async (width: number) => {
    const state = get()
    await state.setSetting('ui.leftPanelWidth', width)
  },
}))

// Helper hook for appearance settings
export const useAppearanceSettings = () => {
  const {
    getAppearanceTheme,
    getAppearanceComponentSize,
    getAppearanceLanguage,
    getResolvedTheme,
    setSetting,
    loading,
  } = useUserSettingsStore()

  return {
    theme: getAppearanceTheme(),
    componentSize: getAppearanceComponentSize(),
    language: getAppearanceLanguage(),
    resolvedTheme: getResolvedTheme(),
    setTheme: (theme: 'light' | 'dark' | 'system') =>
      setSetting('appearance.theme', theme),
    setComponentSize: (componentSize: 'small' | 'medium' | 'large') =>
      setSetting('appearance.componentSize', componentSize),
    setLanguage: (language: 'en' | 'vi') =>
      setSetting('appearance.language', language),
    loading,
  }
}

// Helper hook for UI settings
export const useUISettings = () => {
  const {
    getLeftPanelCollapsed,
    getLeftPanelWidth,
    setLeftPanelCollapsed,
    setLeftPanelWidth,
    loading,
  } = useUserSettingsStore()

  return {
    leftPanelCollapsed: getLeftPanelCollapsed(),
    leftPanelWidth: getLeftPanelWidth(),
    setLeftPanelCollapsed,
    setLeftPanelWidth,
    loading,
  }
}

// Initialize settings on app start
export const initializeUserSettings = async () => {
  const store = useUserSettingsStore.getState()
  // Load both user settings and global language configuration
  await Promise.all([store.loadSettings(), store.loadGlobalLanguage()])
}
