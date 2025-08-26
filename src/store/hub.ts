import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../api/client'
import type { HubAssistant, HubModel } from '../types'
import i18n from '../i18n'

interface HubState {
  models: HubModel[]
  assistants: HubAssistant[]
  hubVersion: string
  lastUpdated: string
  lastActiveTab: string
  currentLocale: string
  modelsInitialized: boolean
  assistantsInitialized: boolean
  modelsLoading: boolean
  assistantsLoading: boolean
  modelsError: string | null
  assistantsError: string | null
}

export const useHubStore = create<HubState>()(
  subscribeWithSelector((_set, _get) => ({
    models: [] as HubModel[],
    assistants: [] as HubAssistant[],
    hubVersion: '',
    lastUpdated: '',
    lastActiveTab: 'models',
    currentLocale: 'en',
    modelsInitialized: false as boolean,
    assistantsInitialized: false as boolean,
    modelsLoading: false as boolean,
    assistantsLoading: false as boolean,
    modelsError: null as string | null,
    assistantsError: null as string | null,
  })),
)

export const loadHubModels = async (locale?: string) => {
  const state = useHubStore.getState()
  const currentLocale = locale || i18n.language || 'en'

  if (state.modelsInitialized || state.modelsLoading) {
    return
  }

  useHubStore.setState({
    modelsLoading: true,
    modelsError: null,
    currentLocale,
  })

  try {
    const models = await ApiClient.Hub.getHubModels({ lang: currentLocale })

    useHubStore.setState({
      models,
      currentLocale,
      modelsInitialized: true,
      modelsLoading: false,
      modelsError: null,
    })

    console.log(
      `Hub models loaded: ${models.length} models - locale: ${currentLocale}`,
    )
  } catch (error) {
    console.error('Hub models loading failed:', error)
    useHubStore.setState({
      modelsLoading: false,
      modelsError: error instanceof Error ? error.message : 'Unknown error',
      modelsInitialized: false,
    })
    throw error
  }
}

export const loadHubAssistants = async (locale?: string) => {
  const state = useHubStore.getState()
  const currentLocale = locale || i18n.language || 'en'

  if (state.assistantsInitialized || state.assistantsLoading) {
    return
  }

  useHubStore.setState({
    assistantsLoading: true,
    assistantsError: null,
    currentLocale,
  })

  try {
    const assistants = await ApiClient.Hub.getHubAssistants({
      lang: currentLocale,
    })

    useHubStore.setState({
      assistants,
      currentLocale,
      assistantsInitialized: true,
      assistantsLoading: false,
      assistantsError: null,
    })

    console.log(
      `Hub assistants loaded: ${assistants.length} assistants - locale: ${currentLocale}`,
    )
  } catch (error) {
    console.error('Hub assistants loading failed:', error)
    useHubStore.setState({
      assistantsLoading: false,
      assistantsError: error instanceof Error ? error.message : 'Unknown error',
      assistantsInitialized: false,
    })
    throw error
  }
}

export const refreshHubModels = async (locale?: string) => {
  const state = useHubStore.getState()
  const currentLocale = locale || state.currentLocale || 'en'

  if (state.modelsLoading) {
    return
  }

  useHubStore.setState({ modelsLoading: true, modelsError: null })

  try {
    await ApiClient.Hub.refreshHubData({ lang: currentLocale })
    const models = await ApiClient.Hub.getHubModels({ lang: currentLocale })

    useHubStore.setState({
      models,
      currentLocale,
      modelsLoading: false,
      modelsError: null,
    })

    console.log(
      `Hub models refreshed: ${models.length} models - locale: ${currentLocale}`,
    )
    return models
  } catch (error) {
    console.error('Hub models refresh failed:', error)
    useHubStore.setState({
      modelsLoading: false,
      modelsError: error instanceof Error ? error.message : 'Unknown error',
    })
    throw error
  }
}

export const refreshHubAssistants = async (locale?: string) => {
  const state = useHubStore.getState()
  const currentLocale = locale || state.currentLocale || 'en'

  if (state.assistantsLoading) {
    return
  }

  useHubStore.setState({ assistantsLoading: true, assistantsError: null })

  try {
    await ApiClient.Hub.refreshHubData({ lang: currentLocale })
    const assistants = await ApiClient.Hub.getHubAssistants({
      lang: currentLocale,
    })

    useHubStore.setState({
      assistants,
      currentLocale,
      assistantsLoading: false,
      assistantsError: null,
    })

    console.log(
      `Hub assistants refreshed: ${assistants.length} assistants - locale: ${currentLocale}`,
    )
    return assistants
  } catch (error) {
    console.error('Hub assistants refresh failed:', error)
    useHubStore.setState({
      assistantsLoading: false,
      assistantsError: error instanceof Error ? error.message : 'Unknown error',
    })
    throw error
  }
}

export const getHubVersion = async (): Promise<string> => {
  try {
    const response = await ApiClient.Hub.getHubVersion()
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

  // Only reload if locale changed and data is initialized
  if (currentState.currentLocale !== newLocale) {
    console.log(
      `Hub locale changed from ${currentState.currentLocale} to ${newLocale}`,
    )

    // Reload models if they were initialized
    if (currentState.modelsInitialized) {
      await loadHubModels(newLocale)
    }

    // Reload assistants if they were initialized
    if (currentState.assistantsInitialized) {
      await loadHubAssistants(newLocale)
    }
  }
}

// Subscribe to i18n language changes
i18n.on('languageChanged', (lng: string) => {
  handleLanguageChange(lng).catch(console.error)
})
