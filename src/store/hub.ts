import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../api/client'
import type { HubModel, HubAssistant, HubData } from '../types/api/hub'

interface HubState {
  models: HubModel[]
  assistants: HubAssistant[]
  hubVersion: string
  lastUpdated: string
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
    initialized: false,
    loading: false,
    error: null,
  })),
)

export const initializeHub = async () => {
  useHubStore.setState({ loading: true, error: null })

  try {
    const hubData = await ApiClient.Hub.getData()

    useHubStore.setState({
      models: hubData.models,
      assistants: hubData.assistants,
      hubVersion: hubData.hub_version,
      lastUpdated: hubData.last_updated,
      initialized: true,
      loading: false,
      error: null,
    })

    console.log(
      `Hub initialized: ${hubData.hub_version} (${hubData.models.length} models, ${hubData.assistants.length} assistants)`,
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

export const refreshHub = async () => {
  useHubStore.setState({ loading: true, error: null })

  try {
    const hubData = await ApiClient.Hub.refresh()
    useHubStore.setState({
      models: hubData.models,
      assistants: hubData.assistants,
      lastUpdated: hubData.last_updated,
      loading: false,
      error: null,
    })

    console.log(
      `Hub refreshed: ${hubData.models.length} models, ${hubData.assistants.length} assistants`,
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
