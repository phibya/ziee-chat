import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../../api/client.ts'
import {
  RAGProvider,
  RAGDatabase,
  CreateRAGProviderRequest,
  UpdateRAGProviderRequest,
  CreateRAGDatabaseRequest,
  UpdateRAGDatabaseRequest,
} from '../../types'

interface AdminRAGProvidersState {
  // Data
  providers: RAGProvider[]
  databasesByProvider: Record<string, RAGDatabase[]>

  // Loading states
  loading: boolean
  creating: boolean
  updating: boolean
  deleting: boolean
  loadingDatabases: Record<string, boolean>
  databaseOperations: Record<string, boolean>

  // Error state
  error: string | null
}

export const useAdminRAGProvidersStore = create<AdminRAGProvidersState>()(
  subscribeWithSelector(
    (): AdminRAGProvidersState => ({
      // Initial state
      providers: [],
      databasesByProvider: {},
      loading: false,
      creating: false,
      updating: false,
      deleting: false,
      loadingDatabases: {},
      databaseOperations: {},
      error: null,
    }),
  ),
)

// Provider actions
export const loadAllRAGProviders = async (): Promise<void> => {
  try {
    useAdminRAGProvidersStore.setState({ loading: true, error: null })
    const response = await ApiClient.Admin.listRAGProviders({})
    useAdminRAGProvidersStore.setState({
      providers: response.providers,
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
): Promise<RAGProvider> => {
  try {
    useAdminRAGProvidersStore.setState({ creating: true, error: null })
    const newProvider = await ApiClient.Admin.createRAGProvider(provider)
    useAdminRAGProvidersStore.setState(state => ({
      providers: [...state.providers, newProvider],
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
  try {
    useAdminRAGProvidersStore.setState({ updating: true, error: null })
    const updatedProvider = await ApiClient.Admin.updateRAGProvider({
      provider_id: id,
      ...provider,
    })
    useAdminRAGProvidersStore.setState(state => ({
      providers: state.providers.map(p => (p.id === id ? updatedProvider : p)),
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
  try {
    useAdminRAGProvidersStore.setState({ deleting: true, error: null })
    await ApiClient.Admin.deleteRAGProvider({ provider_id: id })
    useAdminRAGProvidersStore.setState(state => ({
      providers: state.providers.filter(p => p.id !== id),
      databasesByProvider: Object.fromEntries(
        Object.entries(state.databasesByProvider).filter(
          ([providerId]) => providerId !== id,
        ),
      ),
      deleting: false,
    }))
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

export const cloneExistingRAGProvider = async (
  id: string,
): Promise<RAGProvider> => {
  try {
    useAdminRAGProvidersStore.setState({ creating: true, error: null })
    const clonedProvider = await ApiClient.Admin.cloneRAGProvider({
      provider_id: id,
    })
    useAdminRAGProvidersStore.setState(state => ({
      providers: [...state.providers, clonedProvider],
      creating: false,
    }))
    return clonedProvider
  } catch (error) {
    useAdminRAGProvidersStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to clone RAG provider',
      creating: false,
    })
    throw error
  }
}

// Database actions
export const loadDatabasesForRAGProvider = async (
  providerId: string,
): Promise<void> => {
  try {
    useAdminRAGProvidersStore.setState(state => ({
      loadingDatabases: { ...state.loadingDatabases, [providerId]: true },
      error: null,
    }))

    const databases = await ApiClient.Admin.listRAGProviderDatabases({
      provider_id: providerId,
    })

    useAdminRAGProvidersStore.setState(state => ({
      databasesByProvider: {
        ...state.databasesByProvider,
        [providerId]: databases,
      },
      loadingDatabases: { ...state.loadingDatabases, [providerId]: false },
    }))
  } catch (error) {
    useAdminRAGProvidersStore.setState(state => ({
      error:
        error instanceof Error ? error.message : 'Failed to load RAG databases',
      loadingDatabases: { ...state.loadingDatabases, [providerId]: false },
    }))
    throw error
  }
}

export const addNewDatabaseToRAGProvider = async (
  providerId: string,
  database: CreateRAGDatabaseRequest,
): Promise<void> => {
  try {
    useAdminRAGProvidersStore.setState(state => ({
      loadingDatabases: { ...state.loadingDatabases, [providerId]: true },
      error: null,
    }))

    const newDatabase = await ApiClient.Admin.addDatabaseToRAGProvider({
      provider_id: providerId,
      ...database,
    })

    useAdminRAGProvidersStore.setState(state => ({
      databasesByProvider: {
        ...state.databasesByProvider,
        [providerId]: [
          ...(state.databasesByProvider[providerId] || []),
          newDatabase,
        ],
      },
      loadingDatabases: { ...state.loadingDatabases, [providerId]: false },
    }))
  } catch (error) {
    useAdminRAGProvidersStore.setState(state => ({
      error:
        error instanceof Error ? error.message : 'Failed to add RAG database',
      loadingDatabases: { ...state.loadingDatabases, [providerId]: false },
    }))
    throw error
  }
}

export const updateExistingRAGDatabase = async (
  databaseId: string,
  updates: UpdateRAGDatabaseRequest,
): Promise<void> => {
  try {
    useAdminRAGProvidersStore.setState(state => ({
      databaseOperations: { ...state.databaseOperations, [databaseId]: true },
      error: null,
    }))

    const updatedDatabase = await ApiClient.Admin.updateRAGDatabase({
      database_id: databaseId,
      ...updates,
    })

    useAdminRAGProvidersStore.setState(state => {
      const newDatabasesByProvider = { ...state.databasesByProvider }
      for (const providerId in newDatabasesByProvider) {
        newDatabasesByProvider[providerId] = newDatabasesByProvider[
          providerId
        ].map(database =>
          database.id === databaseId ? updatedDatabase : database,
        )
      }
      return {
        databasesByProvider: newDatabasesByProvider,
        databaseOperations: {
          ...state.databaseOperations,
          [databaseId]: false,
        },
      }
    })
  } catch (error) {
    useAdminRAGProvidersStore.setState(state => ({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to update RAG database',
      databaseOperations: { ...state.databaseOperations, [databaseId]: false },
    }))
    throw error
  }
}

export const deleteExistingRAGDatabase = async (
  databaseId: string,
): Promise<void> => {
  try {
    useAdminRAGProvidersStore.setState(state => ({
      databaseOperations: { ...state.databaseOperations, [databaseId]: true },
      error: null,
    }))

    await ApiClient.Admin.deleteRAGDatabase({ database_id: databaseId })

    useAdminRAGProvidersStore.setState(state => {
      const newDatabasesByProvider = { ...state.databasesByProvider }
      for (const providerId in newDatabasesByProvider) {
        newDatabasesByProvider[providerId] = newDatabasesByProvider[
          providerId
        ].filter(database => database.id !== databaseId)
      }
      return {
        databasesByProvider: newDatabasesByProvider,
        databaseOperations: {
          ...state.databaseOperations,
          [databaseId]: false,
        },
      }
    })
  } catch (error) {
    useAdminRAGProvidersStore.setState(state => ({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to delete RAG database',
      databaseOperations: { ...state.databaseOperations, [databaseId]: false },
    }))
    throw error
  }
}

// Database operations (only for local providers)
export const startRAGDatabase = async (databaseId: string): Promise<void> => {
  try {
    useAdminRAGProvidersStore.setState(state => ({
      databaseOperations: { ...state.databaseOperations, [databaseId]: true },
      error: null,
    }))

    await ApiClient.Admin.startRAGDatabase({ database_id: databaseId })

    useAdminRAGProvidersStore.setState(state => {
      const newDatabasesByProvider = { ...state.databasesByProvider }
      for (const providerId in newDatabasesByProvider) {
        newDatabasesByProvider[providerId] = newDatabasesByProvider[
          providerId
        ].map(database => {
          if (database.id === databaseId) {
            return { ...database, is_active: true }
          }
          return database
        })
      }
      return {
        databasesByProvider: newDatabasesByProvider,
        databaseOperations: {
          ...state.databaseOperations,
          [databaseId]: false,
        },
      }
    })
  } catch (error) {
    useAdminRAGProvidersStore.setState(state => ({
      error:
        error instanceof Error ? error.message : 'Failed to start RAG database',
      databaseOperations: { ...state.databaseOperations, [databaseId]: false },
    }))
    throw error
  }
}

export const stopRAGDatabase = async (databaseId: string): Promise<void> => {
  try {
    useAdminRAGProvidersStore.setState(state => ({
      databaseOperations: { ...state.databaseOperations, [databaseId]: true },
      error: null,
    }))

    await ApiClient.Admin.stopRAGDatabase({ database_id: databaseId })

    useAdminRAGProvidersStore.setState(state => {
      const newDatabasesByProvider = { ...state.databasesByProvider }
      for (const providerId in newDatabasesByProvider) {
        newDatabasesByProvider[providerId] = newDatabasesByProvider[
          providerId
        ].map(database => {
          if (database.id === databaseId) {
            return { ...database, is_active: false }
          }
          return database
        })
      }
      return {
        databasesByProvider: newDatabasesByProvider,
        databaseOperations: {
          ...state.databaseOperations,
          [databaseId]: false,
        },
      }
    })
  } catch (error) {
    useAdminRAGProvidersStore.setState(state => ({
      error:
        error instanceof Error ? error.message : 'Failed to stop RAG database',
      databaseOperations: { ...state.databaseOperations, [databaseId]: false },
    }))
    throw error
  }
}

export const enableRAGDatabase = async (databaseId: string): Promise<void> => {
  try {
    useAdminRAGProvidersStore.setState(state => ({
      databaseOperations: { ...state.databaseOperations, [databaseId]: true },
      error: null,
    }))

    await ApiClient.Admin.enableRAGDatabase({ database_id: databaseId })

    useAdminRAGProvidersStore.setState(state => {
      const newDatabasesByProvider = { ...state.databasesByProvider }
      for (const providerId in newDatabasesByProvider) {
        newDatabasesByProvider[providerId] = newDatabasesByProvider[
          providerId
        ].map(database => {
          if (database.id === databaseId) {
            return { ...database, enabled: true }
          }
          return database
        })
      }
      return {
        databasesByProvider: newDatabasesByProvider,
        databaseOperations: {
          ...state.databaseOperations,
          [databaseId]: false,
        },
      }
    })
  } catch (error) {
    useAdminRAGProvidersStore.setState(state => ({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to enable RAG database',
      databaseOperations: { ...state.databaseOperations, [databaseId]: false },
    }))
    throw error
  }
}

export const disableRAGDatabase = async (databaseId: string): Promise<void> => {
  try {
    useAdminRAGProvidersStore.setState(state => ({
      databaseOperations: { ...state.databaseOperations, [databaseId]: true },
      error: null,
    }))

    await ApiClient.Admin.disableRAGDatabase({ database_id: databaseId })

    useAdminRAGProvidersStore.setState(state => {
      const newDatabasesByProvider = { ...state.databasesByProvider }
      for (const providerId in newDatabasesByProvider) {
        newDatabasesByProvider[providerId] = newDatabasesByProvider[
          providerId
        ].map(database => {
          if (database.id === databaseId) {
            return { ...database, enabled: false }
          }
          return database
        })
      }
      return {
        databasesByProvider: newDatabasesByProvider,
        databaseOperations: {
          ...state.databaseOperations,
          [databaseId]: false,
        },
      }
    })
  } catch (error) {
    useAdminRAGProvidersStore.setState(state => ({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to disable RAG database',
      databaseOperations: { ...state.databaseOperations, [databaseId]: false },
    }))
    throw error
  }
}

// Utility actions
export const clearRAGProvidersError = (): void => {
  useAdminRAGProvidersStore.setState({ error: null })
}

export const findRAGProviderById = (id: string): RAGProvider | undefined => {
  return useAdminRAGProvidersStore.getState().providers.find(p => p.id === id)
}

export const findRAGDatabaseById = (id: string): RAGDatabase | undefined => {
  const state = useAdminRAGProvidersStore.getState()
  for (const databases of Object.values(state.databasesByProvider)) {
    const database = databases.find(d => d.id === id)
    if (database) return database
  }
  return undefined
}
