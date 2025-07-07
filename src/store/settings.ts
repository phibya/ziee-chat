import { create } from 'zustand'
import { persist } from 'zustand/middleware'

export type Theme = 'light' | 'dark' | 'auto'
export type Language = 'en' | 'vi'
export type ComponentSize = 'small' | 'middle' | 'large'

interface SettingsState {
  // Theme settings
  theme: Theme

  // Language settings
  language: Language

  // UI settings
  componentSize: ComponentSize
  leftPanelCollapsed: boolean
  leftPanelWidth: number

  // Chat settings
  autoSave: boolean
  showTimestamps: boolean
  maxTokens: number
  temperature: number

  // API settings
  openaiApiKey: string
  anthropicApiKey: string
  customEndpoint: string
  requestTimeout: number

  // Advanced settings
  enableStreaming: boolean
  enableFunctionCalling: boolean
  debugMode: boolean
  systemPrompt: string

  // Default model
  defaultModel: string

  // Actions
  setTheme: (theme: Theme) => void
  setLanguage: (language: Language) => void
  setComponentSize: (size: ComponentSize) => void
  setLeftPanelCollapsed: (collapsed: boolean) => void
  setLeftPanelWidth: (width: number) => void
  updateSettings: (settings: Partial<SettingsState>) => void
  resetSettings: () => void
}

const defaultSettings = {
  theme: 'light' as Theme,
  language: 'en' as Language,
  componentSize: 'middle' as ComponentSize,
  leftPanelCollapsed: false,
  leftPanelWidth: 280,
  autoSave: true,
  showTimestamps: false,
  maxTokens: 2048,
  temperature: 0.7,
  openaiApiKey: '',
  anthropicApiKey: '',
  customEndpoint: '',
  requestTimeout: 30,
  enableStreaming: true,
  enableFunctionCalling: true,
  debugMode: false,
  systemPrompt: '',
  defaultModel: 'gpt-3.5-turbo',
}

export const useSettingsStore = create<SettingsState>()(
  persist(
    set => ({
      ...defaultSettings,

      setTheme: theme => set({ theme }),

      setLanguage: language => set({ language }),

      setComponentSize: componentSize => set({ componentSize }),

      setLeftPanelCollapsed: leftPanelCollapsed => set({ leftPanelCollapsed }),

      setLeftPanelWidth: leftPanelWidth => set({ leftPanelWidth }),

      updateSettings: settings => set(state => ({ ...state, ...settings })),

      resetSettings: () => set(defaultSettings),
    }),
    {
      name: 'jan-chat-settings',
      version: 1,
      partialize: state => ({
        theme: state.theme,
        language: state.language,
        componentSize: state.componentSize,
        leftPanelCollapsed: state.leftPanelCollapsed,
        leftPanelWidth: state.leftPanelWidth,
        autoSave: state.autoSave,
        showTimestamps: state.showTimestamps,
        maxTokens: state.maxTokens,
        temperature: state.temperature,
        openaiApiKey: state.openaiApiKey,
        anthropicApiKey: state.anthropicApiKey,
        customEndpoint: state.customEndpoint,
        requestTimeout: state.requestTimeout,
        enableStreaming: state.enableStreaming,
        enableFunctionCalling: state.enableFunctionCalling,
        debugMode: state.debugMode,
        systemPrompt: state.systemPrompt,
        defaultModel: state.defaultModel,
      }),
    },
  ),
)

// Helper function to get system theme preference
export const getSystemTheme = (): 'light' | 'dark' => {
  if (typeof window !== 'undefined' && window.matchMedia) {
    return window.matchMedia('(prefers-color-scheme: dark)').matches
      ? 'dark'
      : 'light'
  }
  return 'light'
}

// Helper function to resolve actual theme
export const getResolvedTheme = (theme: Theme): 'light' | 'dark' => {
  if (theme === 'auto') {
    return getSystemTheme()
  }
  return theme
}
