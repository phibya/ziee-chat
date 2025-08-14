import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../../api/client.ts'
import {
  CreateProviderRequest,
  Provider,
  UpdateProviderRequest,
} from '../../types/api/provider.ts'
import { Model, ModelCapabilities } from '../../types'

// Type definitions are now imported from the API types

// Upload-related types moved to localUpload.ts

interface ProviderWithModels extends Provider {
  models: Model[]
}

interface AdminProvidersState {
  // Data
  providers: ProviderWithModels[]

  // Loading states
  loading: boolean
  creating: boolean
  updating: boolean
  deleting: boolean
  modelsLoading: Record<string, boolean> // Track loading state for provider models
  modelOperations: Record<string, boolean> // Track loading state for individual models

  // Error states
  error: string | null
  modelError: Record<string, string> // Track errors for specific providers
}

export const useAdminProvidersStore = create<AdminProvidersState>()(
  subscribeWithSelector(
    (): AdminProvidersState => ({
      // Initial state
      providers: [],
      loading: false,
      creating: false,
      updating: false,
      deleting: false,
      modelsLoading: {},
      modelOperations: {},
      error: null,
      modelError: {},
    }),
  ),
)

// Provider actions
export const loadAllModelProviders = async (): Promise<void> => {
  try {
    useAdminProvidersStore.setState({ loading: true, error: null })
    const response = await ApiClient.Admin.listProviders({})
    const providers = await Promise.all(
      response.providers.map(async provider => {
        // Fetch models for each provider
        const models = await ApiClient.Admin.listProviderModels({
          provider_id: provider.id,
        })
        return { ...provider, models } as ProviderWithModels
      }),
    )
    useAdminProvidersStore.setState({
      providers: providers,
      loading: false,
    })
  } catch (error) {
    useAdminProvidersStore.setState({
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
    useAdminProvidersStore.setState({ creating: true, error: null })
    const newProvider = await ApiClient.Admin.createProvider(provider)
    useAdminProvidersStore.setState(state => ({
      providers: [
        ...state.providers,
        {
          ...newProvider,
          models: [], // Initialize with empty models
        },
      ],
      creating: false,
    }))
    return newProvider
  } catch (error) {
    useAdminProvidersStore.setState({
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
    useAdminProvidersStore.setState({ updating: true, error: null })
    const updatedProvider = await ApiClient.Admin.updateProvider({
      provider_id: id,
      ...provider,
    })
    useAdminProvidersStore.setState(state => ({
      providers: state.providers.map(p =>
        p.id === id
          ? {
              ...updatedProvider,
              models: p.models, // Preserve existing models
            }
          : p,
      ),
      updating: false,
    }))
  } catch (error) {
    useAdminProvidersStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to update provider',
      updating: false,
    })
    throw error
  }
}

export const deleteModelProvider = async (id: string): Promise<void> => {
  try {
    useAdminProvidersStore.setState({ deleting: true, error: null })
    await ApiClient.Admin.deleteProvider({ provider_id: id })

    useAdminProvidersStore.setState(state => {
      // Clean up models loading state and errors for this provider
      const { [id]: removedLoading, ...restModelsLoading } = state.modelsLoading
      const { [id]: removedError, ...restModelError } = state.modelError

      return {
        providers: state.providers.filter(p => p.id !== id),
        modelsLoading: restModelsLoading,
        modelError: restModelError,
        deleting: false,
      }
    })
  } catch (error) {
    useAdminProvidersStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to delete provider',
      deleting: false,
    })
    throw error
  }
}

// Model actions

export const loadModelsForProvider = async (
  providerId: string,
): Promise<void> => {
  try {
    useAdminProvidersStore.setState(state => ({
      modelsLoading: { ...state.modelsLoading, [providerId]: true },
      modelError: { ...state.modelError, [providerId]: '' },
    }))

    const models = await ApiClient.Admin.listProviderModels({
      provider_id: providerId,
    })

    useAdminProvidersStore.setState(state => ({
      providers: state.providers.map(p =>
        p.id === providerId ? { ...p, models } : p,
      ),
      modelsLoading: { ...state.modelsLoading, [providerId]: false },
    }))
  } catch (error) {
    useAdminProvidersStore.setState(state => ({
      modelError: {
        ...state.modelError,
        [providerId]:
          error instanceof Error ? error.message : 'Failed to load models',
      },
      modelsLoading: { ...state.modelsLoading, [providerId]: false },
    }))
    throw error
  }
}

export const addNewModelToProvider = async (
  providerId: string,
  model: {
    name: string
    alias: string
    description?: string
    enabled?: boolean
    capabilities?: ModelCapabilities
    engine_type?: string
  },
): Promise<void> => {
  try {
    useAdminProvidersStore.setState(state => ({
      modelsLoading: { ...state.modelsLoading, [providerId]: true },
      modelError: { ...state.modelError, [providerId]: '' },
    }))

    const newModel = await ApiClient.Admin.addModelToProvider({
      provider_id: providerId,
      engine_type: 'mistralrs', // Default engine type
      ...model,
    })

    useAdminProvidersStore.setState(state => ({
      providers: state.providers.map(p =>
        p.id === providerId ? { ...p, models: [...p.models, newModel] } : p,
      ),
      modelsLoading: { ...state.modelsLoading, [providerId]: false },
    }))
  } catch (error) {
    useAdminProvidersStore.setState(state => ({
      modelError: {
        ...state.modelError,
        [providerId]:
          error instanceof Error ? error.message : 'Failed to add model',
      },
      modelsLoading: { ...state.modelsLoading, [providerId]: false },
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
    useAdminProvidersStore.setState(state => ({
      modelsLoading: { ...state.modelsLoading, [providerId]: true },
      modelError: { ...state.modelError, [providerId]: '' },
    }))

    const { id: _, ...modelData } = data
    const newModel = await ApiClient.Admin.addModelToProvider({
      provider_id: providerId,
      ...modelData,
    } as any)

    useAdminProvidersStore.setState(state => ({
      providers: state.providers.map(p =>
        p.id === providerId ? { ...p, models: [...p.models, newModel] } : p,
      ),
      modelsLoading: { ...state.modelsLoading, [providerId]: false },
    }))

    return newModel
  } catch (error) {
    useAdminProvidersStore.setState(state => ({
      modelError: {
        ...state.modelError,
        [providerId]:
          error instanceof Error ? error.message : 'Failed to add model',
      },
      modelsLoading: { ...state.modelsLoading, [providerId]: false },
    }))
    throw error
  }
}

export const updateExistingModel = async (
  modelId: string,
  updates: { alias?: string; description?: string; enabled?: boolean },
): Promise<void> => {
  try {
    useAdminProvidersStore.setState(state => ({
      modelOperations: { ...state.modelOperations, [modelId]: true },
    }))

    const updatedModel = await ApiClient.Admin.updateModel({
      model_id: modelId,
      ...updates,
    })

    useAdminProvidersStore.setState(state => ({
      providers: state.providers.map(provider => ({
        ...provider,
        models: provider.models.map(model =>
          model.id === modelId ? updatedModel : model,
        ),
      })),
      modelOperations: { ...state.modelOperations, [modelId]: false },
    }))
  } catch (error) {
    useAdminProvidersStore.setState(state => ({
      modelOperations: { ...state.modelOperations, [modelId]: false },
    }))
    throw error
  }
}

export const deleteExistingModel = async (modelId: string): Promise<void> => {
  try {
    useAdminProvidersStore.setState(state => ({
      modelOperations: { ...state.modelOperations, [modelId]: true },
    }))

    await ApiClient.Admin.deleteModel({ model_id: modelId })

    useAdminProvidersStore.setState(state => ({
      providers: state.providers.map(provider => ({
        ...provider,
        models: provider.models.filter(model => model.id !== modelId),
      })),
      modelOperations: { ...state.modelOperations, [modelId]: false },
    }))
  } catch (error) {
    useAdminProvidersStore.setState(state => ({
      modelOperations: { ...state.modelOperations, [modelId]: false },
    }))
    throw error
  }
}

export const startModelExecution = async (modelId: string): Promise<void> => {
  try {
    useAdminProvidersStore.setState(state => ({
      modelOperations: { ...state.modelOperations, [modelId]: true },
    }))

    await ApiClient.Admin.startModel({ model_id: modelId })

    useAdminProvidersStore.setState(state => ({
      providers: state.providers.map(provider => ({
        ...provider,
        models: provider.models.map(model =>
          model.id === modelId ? { ...model, is_active: true } : model,
        ),
      })),
      modelOperations: { ...state.modelOperations, [modelId]: false },
    }))
  } catch (error) {
    useAdminProvidersStore.setState(state => ({
      modelOperations: { ...state.modelOperations, [modelId]: false },
    }))
    throw error
  }
}

export const stopModelExecution = async (modelId: string): Promise<void> => {
  try {
    useAdminProvidersStore.setState(state => ({
      modelOperations: { ...state.modelOperations, [modelId]: true },
    }))

    await ApiClient.Admin.stopModel({ model_id: modelId })

    useAdminProvidersStore.setState(state => ({
      providers: state.providers.map(provider => ({
        ...provider,
        models: provider.models.map(model =>
          model.id === modelId ? { ...model, is_active: false } : model,
        ),
      })),
      modelOperations: { ...state.modelOperations, [modelId]: false },
    }))
  } catch (error) {
    useAdminProvidersStore.setState(state => ({
      modelOperations: { ...state.modelOperations, [modelId]: false },
    }))
    throw error
  }
}

export const enableModelForUse = async (modelId: string): Promise<void> => {
  try {
    useAdminProvidersStore.setState(state => ({
      modelOperations: { ...state.modelOperations, [modelId]: true },
    }))

    await ApiClient.Admin.enableModel({ model_id: modelId })

    useAdminProvidersStore.setState(state => ({
      providers: state.providers.map(provider => ({
        ...provider,
        models: provider.models.map(model =>
          model.id === modelId ? { ...model, enabled: true } : model,
        ),
      })),
      modelOperations: { ...state.modelOperations, [modelId]: false },
    }))
  } catch (error) {
    useAdminProvidersStore.setState(state => ({
      modelOperations: { ...state.modelOperations, [modelId]: false },
    }))
    throw error
  }
}

export const disableModelFromUse = async (modelId: string): Promise<void> => {
  try {
    useAdminProvidersStore.setState(state => ({
      modelOperations: { ...state.modelOperations, [modelId]: true },
    }))

    await ApiClient.Admin.disableModel({ model_id: modelId })

    useAdminProvidersStore.setState(state => ({
      providers: state.providers.map(provider => ({
        ...provider,
        models: provider.models.map(model =>
          model.id === modelId ? { ...model, enabled: false } : model,
        ),
      })),
      modelOperations: { ...state.modelOperations, [modelId]: false },
    }))
  } catch (error) {
    useAdminProvidersStore.setState(state => ({
      modelOperations: { ...state.modelOperations, [modelId]: false },
    }))
    throw error
  }
}

// Utility actions
export const clearProvidersError = (): void => {
  useAdminProvidersStore.setState({ error: null })
}

export const clearModelError = (providerId: string): void => {
  useAdminProvidersStore.setState(state => ({
    modelError: { ...state.modelError, [providerId]: '' },
  }))
}

export const findProviderById = (id: string): Provider | undefined => {
  return useAdminProvidersStore.getState().providers.find(p => p.id === id)
}

export const findModelById = (id: string): Model | undefined => {
  const state = useAdminProvidersStore.getState()
  for (const provider of state.providers) {
    const model = provider.models.find(model => model.id === id)
    if (model) return model
  }
  return undefined
}

// Get models for a specific provider
export const getModelsForProvider = (providerId: string): Model[] => {
  const provider = useAdminProvidersStore
    .getState()
    .providers.find(p => p.id === providerId)
  return provider?.models || []
}

// Get current provider by checking which one has loaded models
export const getCurrentProvider = (): Provider | null => {
  // This function can be used to get the "current" provider if needed
  // For now, we'll return the first provider that has models loaded
  const state = useAdminProvidersStore.getState()
  return state.providers.find(p => p.models.length > 0) || null
}
