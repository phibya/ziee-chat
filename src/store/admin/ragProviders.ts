import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../../api/client'
import {
  RAGProvider,
  RAGInstance,
  CreateRAGProviderRequest,
  UpdateRAGProviderRequest,
  CreateSystemRAGInstanceRequest,
  UpdateRAGInstanceRequest,
} from '../../types/api'

interface RAGProviderWithInstances extends RAGProvider {
  instances: RAGInstance[]
}

interface AdminRAGProvidersState {
  // Data
  providers: RAGProviderWithInstances[]
  isInitialized: boolean

  // Loading states
  loading: boolean
  creating: boolean
  updating: boolean
  deleting: boolean
  instancesLoading: Record<string, boolean>
  instanceOperations: Record<string, boolean>

  // Error states
  error: string | null
  instanceError: Record<string, string>

  __init__: {
    providers: () => Promise<void>
  }
}

export const useAdminRAGProvidersStore = create<AdminRAGProvidersState>()(
  subscribeWithSelector(
    (): AdminRAGProvidersState => ({
      // Initial state
      providers: [],
      isInitialized: false,
      loading: false,
      creating: false,
      updating: false,
      deleting: false,
      instancesLoading: {},
      instanceOperations: {},
      error: null,
      instanceError: {},
      __init__: {
        providers: async () => loadAllRAGProviders(),
      },
    }),
  ),
)

// Provider actions
export const loadAllRAGProviders = async (): Promise<void> => {
  let state = useAdminRAGProvidersStore.getState()
  if (state.isInitialized || state.loading) {
    return
  }
  try {
    useAdminRAGProvidersStore.setState({ loading: true, error: null })
    const response = await ApiClient.Admin.listRagProviders({ per_page: 10000 })
    const providers = await Promise.all(
      response.providers.map(async provider => {
        // Fetch system instances for each provider
        const instancesResponse = await ApiClient.Admin.listSystemRagInstances({
          provider_id: provider.id,
        })
        return {
          ...provider,
          instances: instancesResponse.instances,
        } as RAGProviderWithInstances
      }),
    )
    useAdminRAGProvidersStore.setState({
      providers: providers,
      isInitialized: true,
      loading: false,
    })
  } catch (error) {
    useAdminRAGProvidersStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to load RAG providers',
      loading: false,
    })
    throw error
  }
}

export const createNewRAGProvider = async (
  provider: CreateRAGProviderRequest,
): Promise<RAGProvider | undefined> => {
  const state = useAdminRAGProvidersStore.getState()
  if (state.creating) {
    return
  }

  try {
    useAdminRAGProvidersStore.setState({ creating: true, error: null })
    const newProvider = await ApiClient.Admin.createRagProvider(provider)
    useAdminRAGProvidersStore.setState(state => ({
      providers: [
        ...state.providers,
        {
          ...newProvider,
          instances: [], // Initialize with empty instances
        },
      ],
      creating: false,
    }))
    return newProvider
  } catch (error) {
    useAdminRAGProvidersStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to create RAG provider',
      creating: false,
    })
    throw error
  }
}

export const updateRAGProvider = async (
  id: string,
  provider: UpdateRAGProviderRequest,
): Promise<void> => {
  const state = useAdminRAGProvidersStore.getState()
  if (state.updating) {
    return
  }

  try {
    useAdminRAGProvidersStore.setState({ updating: true, error: null })
    const updatedProvider = await ApiClient.Admin.updateRagProvider({
      provider_id: id,
      ...provider,
    })
    useAdminRAGProvidersStore.setState(state => ({
      providers: state.providers.map(p =>
        p.id === id
          ? {
              ...updatedProvider,
              instances: p.instances, // Preserve existing instances
            }
          : p,
      ),
      updating: false,
    }))
  } catch (error) {
    useAdminRAGProvidersStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to update RAG provider',
      updating: false,
    })
    throw error
  }
}

export const deleteRAGProvider = async (id: string): Promise<void> => {
  const state = useAdminRAGProvidersStore.getState()
  if (state.deleting) {
    return
  }

  try {
    useAdminRAGProvidersStore.setState({ deleting: true, error: null })
    await ApiClient.Admin.deleteRagProvider({ provider_id: id })

    useAdminRAGProvidersStore.setState(state => {
      // Clean up instances loading state and errors for this provider
      const { [id]: _removedLoading, ...restInstancesLoading } =
        state.instancesLoading
      const { [id]: _removedError, ...restInstanceError } = state.instanceError

      return {
        providers: state.providers.filter(p => p.id !== id),
        instancesLoading: restInstancesLoading,
        instanceError: restInstanceError,
        deleting: false,
      }
    })
  } catch (error) {
    useAdminRAGProvidersStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to delete RAG provider',
      deleting: false,
    })
    throw error
  }
}

// System instance actions
export const loadInstancesForProvider = async (
  providerId: string,
): Promise<void> => {
  const state = useAdminRAGProvidersStore.getState()
  if (state.instancesLoading[providerId]) {
    return
  }

  try {
    useAdminRAGProvidersStore.setState(state => ({
      instancesLoading: { ...state.instancesLoading, [providerId]: true },
      instanceError: { ...state.instanceError, [providerId]: '' },
    }))

    const instancesResponse = await ApiClient.Admin.listSystemRagInstances({
      provider_id: providerId,
    })

    useAdminRAGProvidersStore.setState(state => ({
      providers: state.providers.map(p =>
        p.id === providerId
          ? { ...p, instances: instancesResponse.instances }
          : p,
      ),
      instancesLoading: { ...state.instancesLoading, [providerId]: false },
    }))
  } catch (error) {
    useAdminRAGProvidersStore.setState(state => ({
      instanceError: {
        ...state.instanceError,
        [providerId]:
          error instanceof Error ? error.message : 'Failed to load instances',
      },
      instancesLoading: { ...state.instancesLoading, [providerId]: false },
    }))
    throw error
  }
}

export const createSystemRAGInstance = async (
  providerId: string,
  instanceData: CreateSystemRAGInstanceRequest,
): Promise<void> => {
  const state = useAdminRAGProvidersStore.getState()
  if (state.instancesLoading[providerId]) {
    return
  }

  try {
    useAdminRAGProvidersStore.setState(state => ({
      instancesLoading: { ...state.instancesLoading, [providerId]: true },
      instanceError: { ...state.instanceError, [providerId]: '' },
    }))

    const newInstance = await ApiClient.Admin.createSystemRagInstance({
      provider_id: providerId,
      name: instanceData.name,
      description: instanceData.description,
      alias: instanceData.alias || instanceData.name,
      engine_type: instanceData.engine_type,
      embedding_model_id: instanceData.embedding_model_id,
      llm_model_id: instanceData.llm_model_id,
      parameters: instanceData.parameters,
      engine_settings: instanceData.engine_settings,
    })

    useAdminRAGProvidersStore.setState(state => ({
      providers: state.providers.map(p =>
        p.id === providerId
          ? { ...p, instances: [...p.instances, newInstance] }
          : p,
      ),
      instancesLoading: { ...state.instancesLoading, [providerId]: false },
    }))
  } catch (error) {
    useAdminRAGProvidersStore.setState(state => ({
      instanceError: {
        ...state.instanceError,
        [providerId]:
          error instanceof Error ? error.message : 'Failed to create instance',
      },
      instancesLoading: { ...state.instancesLoading, [providerId]: false },
    }))
    throw error
  }
}

export const updateSystemRAGInstance = async (
  instanceId: string,
  updates: UpdateRAGInstanceRequest,
): Promise<void> => {
  const state = useAdminRAGProvidersStore.getState()
  if (state.instanceOperations[instanceId]) {
    return
  }

  try {
    useAdminRAGProvidersStore.setState(state => ({
      instanceOperations: { ...state.instanceOperations, [instanceId]: true },
    }))

    const updatedInstance = await ApiClient.Admin.updateSystemRagInstance({
      instance_id: instanceId,
      ...updates,
    })

    useAdminRAGProvidersStore.setState(state => ({
      providers: state.providers.map(provider => ({
        ...provider,
        instances: provider.instances.map(instance =>
          instance.id === instanceId ? updatedInstance : instance,
        ),
      })),
      instanceOperations: { ...state.instanceOperations, [instanceId]: false },
    }))
  } catch (error) {
    useAdminRAGProvidersStore.setState(state => ({
      instanceOperations: { ...state.instanceOperations, [instanceId]: false },
    }))
    throw error
  }
}

export const deleteSystemRAGInstance = async (
  instanceId: string,
): Promise<void> => {
  const state = useAdminRAGProvidersStore.getState()
  if (state.instanceOperations[instanceId]) {
    return
  }

  try {
    useAdminRAGProvidersStore.setState(state => ({
      instanceOperations: { ...state.instanceOperations, [instanceId]: true },
    }))

    await ApiClient.Admin.deleteSystemRagInstance({ instance_id: instanceId })

    useAdminRAGProvidersStore.setState(state => ({
      providers: state.providers.map(provider => ({
        ...provider,
        instances: provider.instances.filter(
          instance => instance.id !== instanceId,
        ),
      })),
      instanceOperations: { ...state.instanceOperations, [instanceId]: false },
    }))
  } catch (error) {
    useAdminRAGProvidersStore.setState(state => ({
      instanceOperations: { ...state.instanceOperations, [instanceId]: false },
    }))
    throw error
  }
}

export const enableSystemRAGInstance = async (
  instanceId: string,
): Promise<void> => {
  const state = useAdminRAGProvidersStore.getState()
  if (state.instanceOperations[instanceId]) {
    return
  }

  try {
    useAdminRAGProvidersStore.setState(state => ({
      instanceOperations: { ...state.instanceOperations, [instanceId]: true },
    }))

    await ApiClient.Admin.updateSystemRagInstance({
      instance_id: instanceId,
      enabled: true,
    })

    useAdminRAGProvidersStore.setState(state => ({
      providers: state.providers.map(provider => ({
        ...provider,
        instances: provider.instances.map(instance =>
          instance.id === instanceId
            ? { ...instance, enabled: true }
            : instance,
        ),
      })),
      instanceOperations: { ...state.instanceOperations, [instanceId]: false },
    }))
  } catch (error) {
    useAdminRAGProvidersStore.setState(state => ({
      instanceOperations: { ...state.instanceOperations, [instanceId]: false },
    }))
    throw error
  }
}

export const disableSystemRAGInstance = async (
  instanceId: string,
): Promise<void> => {
  const state = useAdminRAGProvidersStore.getState()
  if (state.instanceOperations[instanceId]) {
    return
  }

  try {
    useAdminRAGProvidersStore.setState(state => ({
      instanceOperations: { ...state.instanceOperations, [instanceId]: true },
    }))

    await ApiClient.Admin.updateSystemRagInstance({
      instance_id: instanceId,
      enabled: false,
    })

    useAdminRAGProvidersStore.setState(state => ({
      providers: state.providers.map(provider => ({
        ...provider,
        instances: provider.instances.map(instance =>
          instance.id === instanceId
            ? { ...instance, enabled: false }
            : instance,
        ),
      })),
      instanceOperations: { ...state.instanceOperations, [instanceId]: false },
    }))
  } catch (error) {
    useAdminRAGProvidersStore.setState(state => ({
      instanceOperations: { ...state.instanceOperations, [instanceId]: false },
    }))
    throw error
  }
}

// Utility actions
export const clearRAGProvidersError = (): void => {
  useAdminRAGProvidersStore.setState({ error: null })
}

export const clearRAGInstanceError = (providerId: string): void => {
  useAdminRAGProvidersStore.setState(state => ({
    instanceError: { ...state.instanceError, [providerId]: '' },
  }))
}

export const findRAGProviderById = (id: string): RAGProvider | undefined => {
  return useAdminRAGProvidersStore.getState().providers.find(p => p.id === id)
}

export const findRAGInstanceById = (id: string): RAGInstance | undefined => {
  const state = useAdminRAGProvidersStore.getState()
  for (const provider of state.providers) {
    const instance = provider.instances.find(instance => instance.id === id)
    if (instance) return instance
  }
  return undefined
}

// Get instances for a specific provider
export const getInstancesForProvider = (providerId: string): RAGInstance[] => {
  const provider = useAdminRAGProvidersStore
    .getState()
    .providers.find(p => p.id === providerId)
  return provider?.instances || []
}

// Get current provider by checking which one has loaded instances
export const getCurrentRAGProvider = (): RAGProvider | null => {
  const state = useAdminRAGProvidersStore.getState()
  return state.providers.find(p => p.instances.length > 0) || null
}
