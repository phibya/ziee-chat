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

  // Loading states
  loading: boolean
  initializing: boolean

  // Actions
  loadSettings: () => Promise<void>
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
  getAppearanceFontSize: () => number

  // Helper to get resolved theme (system -> light/dark)
  getResolvedTheme: () => 'light' | 'dark'
}

export const useUserSettingsStore = create<UserSettingsState>((set, get) => ({
  settings: {},
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

  getAppearanceFontSize: () => {
    const state = get()
    return state.getSetting('appearance.fontSize')
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
}))

// Helper hook for appearance settings
export const useAppearanceSettings = () => {
  const {
    getAppearanceTheme,
    getAppearanceFontSize,
    getResolvedTheme,
    setSetting,
    loading,
  } = useUserSettingsStore()

  return {
    theme: getAppearanceTheme(),
    fontSize: getAppearanceFontSize(),
    resolvedTheme: getResolvedTheme(),
    setTheme: (theme: 'light' | 'dark' | 'system') =>
      setSetting('appearance.theme', theme),
    setFontSize: (fontSize: number) =>
      setSetting('appearance.fontSize', fontSize),
    loading,
  }
}

// Initialize settings on app start
export const initializeUserSettings = async () => {
  const store = useUserSettingsStore.getState()
  await store.loadSettings()
}
