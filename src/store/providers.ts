import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../api/client'
import { Model } from '../types'
import { Provider } from '../types'

interface UserProvidersState {
  // Data
  providers: Provider[]
  modelsByProvider: Record<string, Model[]> // Store models by provider ID

  // Loading states
  loading: boolean
  loadingModels: Record<string, boolean> // Track loading state for provider models

  // Error state
  error: string | null
}

export const useUserProvidersStore = create<UserProvidersState>()(
  subscribeWithSelector(
    (): UserProvidersState => ({
      // Initial state
      providers: [],
      modelsByProvider: {},
      loading: false,
      loadingModels: {},
      error: null,
    }),
  ),
)

// Provider actions - now loads active providers only
export const loadUserProviders = async (): Promise<void> => {
  try {
    useUserProvidersStore.setState({ loading: true, error: null })

    const response = await ApiClient.Providers.listEnabledProviders({
      page: 1,
      per_page: 50,
    })

    useUserProvidersStore.setState({
      providers: response.providers,
      loading: false,
    })
  } catch (error) {
    useUserProvidersStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to load providers',
      loading: false,
    })
    throw error
  }
}

// Model actions - now loads active models only
export const loadUserModelsForProvider = async (
  providerId: string,
): Promise<void> => {
  try {
    useUserProvidersStore.setState(state => ({
      loadingModels: { ...state.loadingModels, [providerId]: true },
      error: null,
    }))

    const models = await ApiClient.Models.listEnabledProviderModels({
      provider_id: providerId,
    })

    useUserProvidersStore.setState(state => ({
      modelsByProvider: {
        ...state.modelsByProvider,
        [providerId]: models,
      },
      loadingModels: { ...state.loadingModels, [providerId]: false },
    }))
  } catch (error) {
    useUserProvidersStore.setState(state => ({
      error: error instanceof Error ? error.message : 'Failed to load models',
      loadingModels: { ...state.loadingModels, [providerId]: false },
    }))
    throw error
  }
}

// Load all active providers and all their active models at once
export const loadUserProvidersWithAllModels = async (): Promise<void> => {
  try {
    useUserProvidersStore.setState({ loading: true, error: null })

    // First load all active providers (user endpoints now return active-only data)
    const response = await ApiClient.Providers.listEnabledProviders({
      page: 1,
      per_page: 50,
    })

    const providers = response.providers
    useUserProvidersStore.setState({
      providers,
      loading: false,
    })

    // If no providers, we're done
    if (providers.length === 0) {
      return
    }

    // Set loading state for all providers
    const loadingModels = providers.reduce(
      (acc, provider) => {
        acc[provider.id] = true
        return acc
      },
      {} as Record<string, boolean>,
    )

    useUserProvidersStore.setState(state => ({
      loadingModels: { ...state.loadingModels, ...loadingModels },
    }))

    // Load models for all providers concurrently (now returns active models only)
    const modelPromises = providers.map(async provider => {
      try {
        const models = await ApiClient.Models.listEnabledProviderModels({
          provider_id: provider.id,
        })
        return { providerId: provider.id, models }
      } catch (error) {
        console.error(
          `Failed to load models for provider ${provider.id}:`,
          error,
        )
        return { providerId: provider.id, models: [], error }
      }
    })

    const results = await Promise.allSettled(modelPromises)

    // Update state with all results
    useUserProvidersStore.setState(state => {
      const newModelsByProvider = { ...state.modelsByProvider }
      const newLoadingModels = { ...state.loadingModels }

      results.forEach(result => {
        if (result.status === 'fulfilled' && result.value) {
          const { providerId, models } = result.value
          newModelsByProvider[providerId] = models
          newLoadingModels[providerId] = false
        } else if (result.status === 'rejected') {
          // Handle individual provider failures gracefully
          console.error('Provider model loading failed:', result.reason)
        }
      })

      return {
        modelsByProvider: newModelsByProvider,
        loadingModels: newLoadingModels,
      }
    })
  } catch (error) {
    useUserProvidersStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to load providers and models',
      loading: false,
      loadingModels: {},
    })
    throw error
  }
}

// Utility actions
export const clearUserProvidersError = (): void => {
  useUserProvidersStore.setState({ error: null })
}

export const findUserProviderById = (id: string): Provider | undefined => {
  return useUserProvidersStore.getState().providers.find(p => p.id === id)
}

export const findUserModelById = (id: string): Model | undefined => {
  const state = useUserProvidersStore.getState()
  for (const models of Object.values(state.modelsByProvider)) {
    const model = models.find(m => m.id === id)
    if (model) return model
  }
  return undefined
}
