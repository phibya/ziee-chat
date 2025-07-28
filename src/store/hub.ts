import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../api/client'
import type { HubModel, HubAssistant } from '../types/api/hub'
import i18n from '../i18n'

interface HubState {
  models: HubModel[]
  assistants: HubAssistant[]
  hubVersion: string
  lastUpdated: string
  lastActiveTab: string
  currentLocale: string
  initialized: boolean
  loading: boolean
  error: string | null
}

export const useHubStore = create<HubState>()(
  subscribeWithSelector((_set, _get) => ({
    models: [],
    assistants: [],
    hubVersion: '',
    lastUpdated: '',
    lastActiveTab: 'models',
    currentLocale: 'en',
    initialized: false,
    loading: false,
    error: null,
  })),
)

export const initializeHub = async (locale?: string) => {
  const currentLocale = locale || i18n.language || 'en'
  useHubStore.setState({ loading: true, error: null, currentLocale })

  try {
    const hubData = await ApiClient.Hub.getData({ lang: currentLocale })

    useHubStore.setState({
      models: hubData.models,
      assistants: hubData.assistants,
      hubVersion: hubData.hub_version,
      lastUpdated: hubData.last_updated,
      currentLocale,
      initialized: true,
      loading: false,
      error: null,
    })

    console.log(
      `Hub initialized: ${hubData.hub_version} (${hubData.models.length} models, ${hubData.assistants.length} assistants) - locale: ${currentLocale}`,
    )
  } catch (error) {
    console.error('Hub initialization failed:', error)
    useHubStore.setState({
      loading: false,
      error: error instanceof Error ? error.message : 'Unknown error',
      initialized: false,
    })
    throw error
  }
}

export const refreshHub = async (locale?: string) => {
  const currentLocale = locale || useHubStore.getState().currentLocale || 'en'
  useHubStore.setState({ loading: true, error: null })

  try {
    const hubData = await ApiClient.Hub.refresh({ lang: currentLocale })
    useHubStore.setState({
      models: hubData.models,
      assistants: hubData.assistants,
      lastUpdated: hubData.last_updated,
      currentLocale,
      loading: false,
      error: null,
    })

    console.log(
      `Hub refreshed: ${hubData.models.length} models, ${hubData.assistants.length} assistants - locale: ${currentLocale}`,
    )
    return hubData
  } catch (error) {
    console.error('Hub refresh failed:', error)
    useHubStore.setState({
      loading: false,
      error: error instanceof Error ? error.message : 'Unknown error',
    })
    throw error
  }
}

export const getHubVersion = async (): Promise<string> => {
  try {
    const response = await ApiClient.Hub.getVersion()
    return response.hub_version
  } catch (error) {
    console.error('Failed to get hub version:', error)
    throw error
  }
}

// Helper functions
export const getModelsByCategory = (
  models: HubModel[],
): Record<string, HubModel[]> => {
  const categories: Record<string, HubModel[]> = {}

  models.forEach(model => {
    const category = model.capabilities?.vision
      ? 'Vision'
      : model.capabilities?.tools
        ? 'Tools'
        : 'Text'

    if (!categories[category]) {
      categories[category] = []
    }
    categories[category].push(model)
  })

  return categories
}

export const getAssistantsByCategory = (
  assistants: HubAssistant[],
): Record<string, HubAssistant[]> => {
  const categories: Record<string, HubAssistant[]> = {}

  assistants.forEach(assistant => {
    const category =
      assistant.category.charAt(0).toUpperCase() + assistant.category.slice(1)

    if (!categories[category]) {
      categories[category] = []
    }
    categories[category].push(assistant)
  })

  return categories
}

export const searchModels = (models: HubModel[], query: string): HubModel[] => {
  if (!query.trim()) return models

  const searchTerm = query.toLowerCase()
  return models.filter(
    model =>
      model.name.toLowerCase().includes(searchTerm) ||
      model.alias.toLowerCase().includes(searchTerm) ||
      model.description?.toLowerCase().includes(searchTerm) ||
      model.tags.some(tag => tag.toLowerCase().includes(searchTerm)),
  )
}

export const searchAssistants = (
  assistants: HubAssistant[],
  query: string,
): HubAssistant[] => {
  if (!query.trim()) return assistants

  const searchTerm = query.toLowerCase()
  return assistants.filter(
    assistant =>
      assistant.name.toLowerCase().includes(searchTerm) ||
      assistant.description?.toLowerCase().includes(searchTerm) ||
      assistant.category.toLowerCase().includes(searchTerm) ||
      assistant.tags.some(tag => tag.toLowerCase().includes(searchTerm)),
  )
}

// Set the last active tab
export const setHubActiveTab = (tab: string) => {
  useHubStore.setState({ lastActiveTab: tab })
}

// Handle language change
export const handleLanguageChange = async (newLocale: string) => {
  const currentState = useHubStore.getState()
  
  // Only reload if locale changed and hub is initialized
  if (currentState.initialized && currentState.currentLocale !== newLocale) {
    console.log(`Hub locale changed from ${currentState.currentLocale} to ${newLocale}`)
    await initializeHub(newLocale)
  }
}

// Subscribe to i18n language changes
i18n.on('languageChanged', (lng: string) => {
  handleLanguageChange(lng).catch(console.error)
})
