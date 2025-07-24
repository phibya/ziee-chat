import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../api/client'
import { Model, ModelCapabilities } from '../types/api/model'
import {
  CreateProviderRequest,
  Provider,
  UpdateProviderRequest,
} from '../types/api/provider'

// Type definitions are now imported from the API types

// Upload-related types moved to localUpload.ts

interface ProvidersState {
  // Data
  providers: Provider[]
  modelsByProvider: Record<string, Model[]> // Store models by provider ID

  // Loading states
  loading: boolean
  creating: boolean
  updating: boolean
  deleting: boolean
  loadingModels: Record<string, boolean> // Track loading state for provider models
  modelOperations: Record<string, boolean> // Track loading state for individual models

  // Upload states moved to localUpload.ts

  // Error state
  error: string | null
}

export const useProvidersStore = create<ProvidersState>()(
  subscribeWithSelector(
    (): ProvidersState => ({
      // Initial state
      providers: [],
      modelsByProvider: {},
      loading: false,
      creating: false,
      updating: false,
      deleting: false,
      loadingModels: {},
      modelOperations: {},
      error: null,
    }),
  ),
)

// Provider actions
export const loadAllModelProviders = async (): Promise<void> => {
  try {
    useProvidersStore.setState({ loading: true, error: null })
    const response = await ApiClient.Providers.list({})
    useProvidersStore.setState({
      providers: response.providers,
      loading: false,
    })
  } catch (error) {
    useProvidersStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to load providers',
      loading: false,
    })
    throw error
  }
}

export const createNewModelProvider = async (
  provider: CreateProviderRequest,
): Promise<Provider> => {
  try {
    useProvidersStore.setState({ creating: true, error: null })
    const newProvider = await ApiClient.Providers.create(provider)
    useProvidersStore.setState(state => ({
      providers: [...state.providers, newProvider],
      creating: false,
    }))
    return newProvider
  } catch (error) {
    useProvidersStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to create provider',
      creating: false,
    })
    throw error
  }
}

export const updateModelProvider = async (
  id: string,
  provider: UpdateProviderRequest,
): Promise<void> => {
  try {
    useProvidersStore.setState({ updating: true, error: null })
    const updatedProvider = await ApiClient.Providers.update({
      provider_id: id,
      ...provider,
    })
    useProvidersStore.setState(state => ({
      providers: state.providers.map(p => (p.id === id ? updatedProvider : p)),
      updating: false,
    }))
  } catch (error) {
    useProvidersStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to update provider',
      updating: false,
    })
    throw error
  }
}

export const deleteModelProvider = async (id: string): Promise<void> => {
  try {
    useProvidersStore.setState({ deleting: true, error: null })
    await ApiClient.Providers.delete({ provider_id: id })
    useProvidersStore.setState(state => ({
      providers: state.providers.filter(p => p.id !== id),
      modelsByProvider: Object.fromEntries(
        Object.entries(state.modelsByProvider).filter(
          ([providerId]) => providerId !== id,
        ),
      ),
      deleting: false,
    }))
  } catch (error) {
    useProvidersStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to delete provider',
      deleting: false,
    })
    throw error
  }
}

export const cloneExistingProvider = async (id: string): Promise<Provider> => {
  try {
    useProvidersStore.setState({ creating: true, error: null })
    const clonedProvider = await ApiClient.Providers.clone({ provider_id: id })
    useProvidersStore.setState(state => ({
      providers: [...state.providers, clonedProvider],
      creating: false,
    }))
    return clonedProvider
  } catch (error) {
    useProvidersStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to clone provider',
      creating: false,
    })
    throw error
  }
}

// Model actions
export const loadModelsForProvider = async (
  providerId: string,
): Promise<void> => {
  try {
    useProvidersStore.setState(state => ({
      loadingModels: { ...state.loadingModels, [providerId]: true },
      error: null,
    }))

    const models = await ApiClient.Providers.listModels({
      provider_id: providerId,
    })

    useProvidersStore.setState(state => ({
      modelsByProvider: {
        ...state.modelsByProvider,
        [providerId]: models,
      },
      loadingModels: { ...state.loadingModels, [providerId]: false },
    }))
  } catch (error) {
    useProvidersStore.setState(state => ({
      error: error instanceof Error ? error.message : 'Failed to load models',
      loadingModels: { ...state.loadingModels, [providerId]: false },
    }))
    throw error
  }
}

// Alias for compatibility
export const loadModels = loadModelsForProvider

export const addNewModelToProvider = async (
  providerId: string,
  model: {
    name: string
    alias: string
    description?: string
    enabled?: boolean
    capabilities?: ModelCapabilities
  },
): Promise<void> => {
  try {
    useProvidersStore.setState(state => ({
      loadingModels: { ...state.loadingModels, [providerId]: true },
      error: null,
    }))

    const newModel = await ApiClient.Providers.addModel({
      provider_id: providerId,
      ...model,
    })

    useProvidersStore.setState(state => ({
      modelsByProvider: {
        ...state.modelsByProvider,
        [providerId]: [...(state.modelsByProvider[providerId] || []), newModel],
      },
      loadingModels: { ...state.loadingModels, [providerId]: false },
    }))
  } catch (error) {
    useProvidersStore.setState(state => ({
      error: error instanceof Error ? error.message : 'Failed to add model',
      loadingModels: { ...state.loadingModels, [providerId]: false },
    }))
    throw error
  }
}

// Legacy compatibility
export const addNewModel = async (
  providerId: string,
  data: Partial<Model>,
): Promise<Model> => {
  try {
    useProvidersStore.setState(state => ({
      loadingModels: { ...state.loadingModels, [providerId]: true },
      error: null,
    }))

    const { id: _, ...modelData } = data
    const newModel = await ApiClient.Providers.addModel({
      provider_id: providerId,
      ...modelData,
    } as any)

    useProvidersStore.setState(state => ({
      modelsByProvider: {
        ...state.modelsByProvider,
        [providerId]: [...(state.modelsByProvider[providerId] || []), newModel],
      },
      loadingModels: { ...state.loadingModels, [providerId]: false },
    }))

    return newModel
  } catch (error) {
    useProvidersStore.setState(state => ({
      error: error instanceof Error ? error.message : 'Failed to add model',
      loadingModels: { ...state.loadingModels, [providerId]: false },
    }))
    throw error
  }
}

export const updateExistingModel = async (
  modelId: string,
  updates: { alias?: string; description?: string; enabled?: boolean },
): Promise<void> => {
  try {
    useProvidersStore.setState(state => ({
      modelOperations: { ...state.modelOperations, [modelId]: true },
      error: null,
    }))

    const updatedModel = await ApiClient.Models.update({
      model_id: modelId,
      ...updates,
    })

    useProvidersStore.setState(state => {
      const newModelsByProvider = { ...state.modelsByProvider }
      for (const providerId in newModelsByProvider) {
        newModelsByProvider[providerId] = newModelsByProvider[providerId].map(
          model => (model.id === modelId ? updatedModel : model),
        )
      }
      return {
        modelsByProvider: newModelsByProvider,
        modelOperations: { ...state.modelOperations, [modelId]: false },
      }
    })
  } catch (error) {
    useProvidersStore.setState(state => ({
      error: error instanceof Error ? error.message : 'Failed to update model',
      modelOperations: { ...state.modelOperations, [modelId]: false },
    }))
    throw error
  }
}

export const deleteExistingModel = async (modelId: string): Promise<void> => {
  try {
    useProvidersStore.setState(state => ({
      modelOperations: { ...state.modelOperations, [modelId]: true },
      error: null,
    }))

    await ApiClient.Models.delete({ model_id: modelId })

    useProvidersStore.setState(state => {
      const newModelsByProvider = { ...state.modelsByProvider }
      for (const providerId in newModelsByProvider) {
        newModelsByProvider[providerId] = newModelsByProvider[
          providerId
        ].filter(model => model.id !== modelId)
      }
      return {
        modelsByProvider: newModelsByProvider,
        modelOperations: { ...state.modelOperations, [modelId]: false },
      }
    })
  } catch (error) {
    useProvidersStore.setState(state => ({
      error: error instanceof Error ? error.message : 'Failed to delete model',
      modelOperations: { ...state.modelOperations, [modelId]: false },
    }))
    throw error
  }
}

export const startModelExecution = async (modelId: string): Promise<void> => {
  try {
    useProvidersStore.setState(state => ({
      modelOperations: { ...state.modelOperations, [modelId]: true },
      error: null,
    }))

    const response = await ApiClient.Models.start({ model_id: modelId })

    useProvidersStore.setState(state => {
      const newModelsByProvider = { ...state.modelsByProvider }
      for (const providerId in newModelsByProvider) {
        newModelsByProvider[providerId] = newModelsByProvider[providerId].map(
          model => {
            if (model.id === modelId) {
              // If response is a success object, update model status; otherwise use response as model
              if ('success' in response && 'message' in response) {
                return { ...model, status: 'running' }
              }
              return response as Model
            }
            return model
          },
        )
      }
      return {
        modelsByProvider: newModelsByProvider,
        modelOperations: { ...state.modelOperations, [modelId]: false },
      }
    })
  } catch (error) {
    useProvidersStore.setState(state => ({
      error: error instanceof Error ? error.message : 'Failed to start model',
      modelOperations: { ...state.modelOperations, [modelId]: false },
    }))
    throw error
  }
}

export const stopModelExecution = async (modelId: string): Promise<void> => {
  try {
    useProvidersStore.setState(state => ({
      modelOperations: { ...state.modelOperations, [modelId]: true },
      error: null,
    }))

    const response = await ApiClient.Models.stop({ model_id: modelId })

    useProvidersStore.setState(state => {
      const newModelsByProvider = { ...state.modelsByProvider }
      for (const providerId in newModelsByProvider) {
        newModelsByProvider[providerId] = newModelsByProvider[providerId].map(
          model => {
            if (model.id === modelId) {
              // If response is a success object, update model status; otherwise use response as model
              if ('success' in response && 'message' in response) {
                return { ...model, status: 'stopped' }
              }
              return response as Model
            }
            return model
          },
        )
      }
      return {
        modelsByProvider: newModelsByProvider,
        modelOperations: { ...state.modelOperations, [modelId]: false },
      }
    })
  } catch (error) {
    useProvidersStore.setState(state => ({
      error: error instanceof Error ? error.message : 'Failed to stop model',
      modelOperations: { ...state.modelOperations, [modelId]: false },
    }))
    throw error
  }
}

export const enableModelForUse = async (modelId: string): Promise<void> => {
  try {
    useProvidersStore.setState(state => ({
      modelOperations: { ...state.modelOperations, [modelId]: true },
      error: null,
    }))

    const response = await ApiClient.Models.enable({ model_id: modelId })

    useProvidersStore.setState(state => {
      const newModelsByProvider = { ...state.modelsByProvider }
      for (const providerId in newModelsByProvider) {
        newModelsByProvider[providerId] = newModelsByProvider[providerId].map(
          model => {
            if (model.id === modelId) {
              // If response is a success object, update model status; otherwise use response as model
              if ('success' in response && 'message' in response) {
                return { ...model, enabled: true }
              }
              return response as Model
            }
            return model
          },
        )
      }
      return {
        modelsByProvider: newModelsByProvider,
        modelOperations: { ...state.modelOperations, [modelId]: false },
      }
    })
  } catch (error) {
    useProvidersStore.setState(state => ({
      error: error instanceof Error ? error.message : 'Failed to enable model',
      modelOperations: { ...state.modelOperations, [modelId]: false },
    }))
    throw error
  }
}

export const disableModelFromUse = async (modelId: string): Promise<void> => {
  try {
    useProvidersStore.setState(state => ({
      modelOperations: { ...state.modelOperations, [modelId]: true },
      error: null,
    }))

    const response = await ApiClient.Models.disable({ model_id: modelId })

    useProvidersStore.setState(state => {
      const newModelsByProvider = { ...state.modelsByProvider }
      for (const providerId in newModelsByProvider) {
        newModelsByProvider[providerId] = newModelsByProvider[providerId].map(
          model => {
            if (model.id === modelId) {
              // If response is a success object, update model status; otherwise use response as model
              if ('success' in response && 'message' in response) {
                return { ...model, enabled: false }
              }
              return response as Model
            }
            return model
          },
        )
      }
      return {
        modelsByProvider: newModelsByProvider,
        modelOperations: { ...state.modelOperations, [modelId]: false },
      }
    })
  } catch (error) {
    useProvidersStore.setState(state => ({
      error: error instanceof Error ? error.message : 'Failed to disable model',
      modelOperations: { ...state.modelOperations, [modelId]: false },
    }))
    throw error
  }
}

// Upload functionality moved to localUpload.ts

// Utility actions
export const clearProvidersError = (): void => {
  useProvidersStore.setState({ error: null })
}

// Upload cancellation moved to localUpload.ts

export const findProviderById = (id: string): Provider | undefined => {
  return useProvidersStore.getState().providers.find(p => p.id === id)
}

export const findModelById = (id: string): Model | undefined => {
  const state = useProvidersStore.getState()
  for (const models of Object.values(state.modelsByProvider)) {
    const model = models.find(m => m.id === id)
    if (model) return model
  }
  return undefined
}
