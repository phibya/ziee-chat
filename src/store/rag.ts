import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../api/client'
import type {
  RAGInstance,
  RAGProvider,
  UpdateRAGInstanceRequest,
} from '../types/api'

interface RagState {
  // Data
  ragInstances: RAGInstance[]
  creatableProviders: RAGProvider[]
  isInitialized: boolean

  // Loading states
  loading: boolean
  creating: boolean
  updating: boolean
  deleting: boolean

  // System instances toggle
  includeSystemInstances: boolean

  // Error state
  error: string | null
}

export const useRAGStore = create<RagState>()(
  subscribeWithSelector(
    (): RagState => ({
      // Initial state
      ragInstances: [],
      creatableProviders: [],
      isInitialized: false,
      loading: false,
      creating: false,
      updating: false,
      deleting: false,
      includeSystemInstances: false,
      error: null,
    }),
  ),
)

// Store methods - defined OUTSIDE the store definition

// RAG list actions
export const loadAllUserRAGInstances = async (
  includeSystem?: boolean,
): Promise<void> => {
  const state = useRAGStore.getState()
  const shouldIncludeSystem = includeSystem ?? state.includeSystemInstances

  if (state.loading) {
    return
  }
  try {
    useRAGStore.setState({ loading: true, error: null })

    const [instancesResponse, providersResponse] = await Promise.all([
      ApiClient.Rag.listInstances({ include_system: shouldIncludeSystem }),
      ApiClient.Rag.listCreatableProviders(),
    ])

    useRAGStore.setState({
      ragInstances: instancesResponse.instances || [],
      creatableProviders: providersResponse || [],
      includeSystemInstances: shouldIncludeSystem,
      isInitialized: true,
      loading: false,
    })
  } catch (error) {
    useRAGStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to load RAG instances',
      loading: false,
    })
    throw error
  }
}

// New toggle function
export const toggleSystemInstances = async (): Promise<void> => {
  const currentState = useRAGStore.getState()
  const newIncludeSystem = !currentState.includeSystemInstances

  // Reset state and reload with new setting
  useRAGStore.setState({
    isInitialized: false,
    includeSystemInstances: newIncludeSystem,
  })

  await loadAllUserRAGInstances(newIncludeSystem)
}

export const createRAGInstance = async (data: {
  name: string
  description: string
  provider_id: string
  alias: string
  engine_type: any
}): Promise<RAGInstance> => {
  try {
    useRAGStore.setState({ creating: true, error: null })

    const ragInstance = await ApiClient.Rag.createInstance(data)

    useRAGStore.setState(state => ({
      ragInstances: [...state.ragInstances, ragInstance],
      creating: false,
    }))

    return ragInstance
  } catch (error) {
    useRAGStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to create RAG instance',
      creating: false,
    })
    throw error
  }
}

export const updateRAGInstanceInList = async (
  id: string,
  data: UpdateRAGInstanceRequest,
): Promise<RAGInstance> => {
  try {
    useRAGStore.setState({ updating: true, error: null })

    const ragInstance = await ApiClient.Rag.updateInstance({
      instance_id: id,
      ...data,
    })

    useRAGStore.setState(state => ({
      ragInstances: state.ragInstances.map(r =>
        r.id === id ? ragInstance : r,
      ),
      updating: false,
    }))

    return ragInstance
  } catch (error) {
    useRAGStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to update RAG instance',
      updating: false,
    })
    throw error
  }
}

export const deleteRAGInstance = async (id: string): Promise<void> => {
  try {
    useRAGStore.setState({ deleting: true, error: null })

    await ApiClient.Rag.deleteInstance({ instance_id: id })

    useRAGStore.setState(state => ({
      ragInstances: state.ragInstances.filter(r => r.id !== id),
      deleting: false,
    }))
  } catch (error) {
    useRAGStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to delete RAG instance',
      deleting: false,
    })
    throw error
  }
}

export const toggleRAGInstanceActivate = async (
  id: string,
): Promise<RAGInstance> => {
  try {
    useRAGStore.setState({ updating: true, error: null })

    const ragInstance = await ApiClient.Rag.toggleInstanceActivate({
      instance_id: id,
    })

    useRAGStore.setState(state => ({
      ragInstances: state.ragInstances.map(r =>
        r.id === id ? ragInstance : r,
      ),
      updating: false,
    }))

    return ragInstance
  } catch (error) {
    useRAGStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to toggle RAG instance activate status',
      updating: false,
    })
    throw error
  }
}

// Utility actions
export const clearRAGStoreError = (): void => {
  useRAGStore.setState({ error: null })
}

export const resetRAGStore = (): void => {
  useRAGStore.setState({
    ragInstances: [],
    creatableProviders: [],
    isInitialized: false,
    loading: false,
    creating: false,
    updating: false,
    deleting: false,
    includeSystemInstances: false,
    error: null,
  })
}

// Helper/utility functions
export const searchRAGInstances = (
  instances: RAGInstance[],
  query: string,
): RAGInstance[] => {
  if (!query.trim()) return instances

  const searchTerm = query.toLowerCase()
  return instances.filter(
    instance =>
      instance.name.toLowerCase().includes(searchTerm) ||
      instance.description?.toLowerCase().includes(searchTerm),
  )
}
