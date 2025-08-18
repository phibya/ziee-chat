import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../../api/client.ts'
import {
  CreateRAGRepositoryRequest,
  DownloadRAGDatabaseFromRepositoryRequest,
  RAGDatabase,
  RAGRepository,
  UpdateRAGRepositoryRequest,
} from '../../types'

interface AdminRAGRepositoriesState {
  // Data
  repositories: RAGRepository[]
  availableDatabasesByRepository: Record<string, RAGDatabase[]>

  // Loading states
  loading: boolean
  creating: boolean
  updating: boolean
  deleting: boolean
  loadingDatabases: Record<string, boolean>
  testingConnection: Record<string, boolean>
  downloading: Record<string, boolean>

  // Error state
  error: string | null
}

export const useAdminRAGRepositoriesStore = create<AdminRAGRepositoriesState>()(
  subscribeWithSelector(
    (): AdminRAGRepositoriesState => ({
      // Initial state
      repositories: [],
      availableDatabasesByRepository: {},
      loading: false,
      creating: false,
      updating: false,
      deleting: false,
      loadingDatabases: {},
      testingConnection: {},
      downloading: {},
      error: null,
    }),
  ),
)

// Repository actions
export const loadAllRAGRepositories = async (): Promise<void> => {
  try {
    useAdminRAGRepositoriesStore.setState({ loading: true, error: null })
    const response = await ApiClient.Admin.listRAGRepositories({})
    useAdminRAGRepositoriesStore.setState({
      repositories: response.repositories,
      loading: false,
    })
  } catch (error) {
    useAdminRAGRepositoriesStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to load RAG repositories',
      loading: false,
    })
    throw error
  }
}

export const createNewRAGRepository = async (
  repository: CreateRAGRepositoryRequest,
): Promise<RAGRepository> => {
  try {
    useAdminRAGRepositoriesStore.setState({ creating: true, error: null })
    const newRepository = await ApiClient.Admin.createRAGRepository(repository)
    useAdminRAGRepositoriesStore.setState(state => ({
      repositories: [...state.repositories, newRepository],
      creating: false,
    }))
    return newRepository
  } catch (error) {
    useAdminRAGRepositoriesStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to create RAG repository',
      creating: false,
    })
    throw error
  }
}

export const updateRAGRepository = async (
  id: string,
  repository: UpdateRAGRepositoryRequest,
): Promise<void> => {
  try {
    useAdminRAGRepositoriesStore.setState({ updating: true, error: null })
    const updatedRepository = await ApiClient.Admin.updateRAGRepository({
      repository_id: id,
      ...repository,
    })
    useAdminRAGRepositoriesStore.setState(state => ({
      repositories: state.repositories.map(r =>
        r.id === id ? updatedRepository : r,
      ),
      updating: false,
    }))
  } catch (error) {
    useAdminRAGRepositoriesStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to update RAG repository',
      updating: false,
    })
    throw error
  }
}

export const deleteRAGRepository = async (id: string): Promise<void> => {
  try {
    useAdminRAGRepositoriesStore.setState({ deleting: true, error: null })
    await ApiClient.Admin.deleteRAGRepository({ repository_id: id })
    useAdminRAGRepositoriesStore.setState(state => ({
      repositories: state.repositories.filter(r => r.id !== id),
      availableDatabasesByRepository: Object.fromEntries(
        Object.entries(state.availableDatabasesByRepository).filter(
          ([repositoryId]) => repositoryId !== id,
        ),
      ),
      deleting: false,
    }))
  } catch (error) {
    useAdminRAGRepositoriesStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to delete RAG repository',
      deleting: false,
    })
    throw error
  }
}

export const testRAGRepositoryConnection = async (
  id: string,
): Promise<void> => {
  try {
    useAdminRAGRepositoriesStore.setState(state => ({
      testingConnection: { ...state.testingConnection, [id]: true },
      error: null,
    }))

    await ApiClient.Admin.testRAGRepositoryConnection({
      repository_id: id,
    })

    useAdminRAGRepositoriesStore.setState(state => ({
      testingConnection: { ...state.testingConnection, [id]: false },
    }))
  } catch (error) {
    useAdminRAGRepositoriesStore.setState(state => ({
      error:
        error instanceof Error
          ? error.message
          : 'Repository connection test failed',
      testingConnection: { ...state.testingConnection, [id]: false },
    }))
    throw error
  }
}

// Database actions
export const loadAvailableDatabasesFromRepository = async (
  repositoryId: string,
): Promise<void> => {
  try {
    useAdminRAGRepositoriesStore.setState(state => ({
      loadingDatabases: { ...state.loadingDatabases, [repositoryId]: true },
      error: null,
    }))

    const databases = await ApiClient.Admin.listRAGRepositoryDatabases({
      repository_id: repositoryId,
    })

    useAdminRAGRepositoriesStore.setState(state => ({
      availableDatabasesByRepository: {
        ...state.availableDatabasesByRepository,
        [repositoryId]: databases,
      },
      loadingDatabases: { ...state.loadingDatabases, [repositoryId]: false },
    }))
  } catch (error) {
    useAdminRAGRepositoriesStore.setState(state => ({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to load available RAG databases',
      loadingDatabases: { ...state.loadingDatabases, [repositoryId]: false },
    }))
    throw error
  }
}

export const downloadRAGDatabaseFromRepository = async (
  request: DownloadRAGDatabaseFromRepositoryRequest,
): Promise<void> => {
  try {
    const downloadKey = `${request.repository_id}-${request.database_id}`
    useAdminRAGRepositoriesStore.setState(state => ({
      downloading: { ...state.downloading, [downloadKey]: true },
      error: null,
    }))

    await ApiClient.Admin.downloadRAGDatabaseFromRepository(request)

    useAdminRAGRepositoriesStore.setState(state => ({
      downloading: { ...state.downloading, [downloadKey]: false },
    }))
  } catch (error) {
    const downloadKey = `${request.repository_id}-${request.database_id}`
    useAdminRAGRepositoriesStore.setState(state => ({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to download RAG database',
      downloading: { ...state.downloading, [downloadKey]: false },
    }))
    throw error
  }
}

// Utility actions
export const clearRAGRepositoriesError = (): void => {
  useAdminRAGRepositoriesStore.setState({ error: null })
}

export const findRAGRepositoryById = (
  id: string,
): RAGRepository | undefined => {
  return useAdminRAGRepositoriesStore
    .getState()
    .repositories.find(r => r.id === id)
}

export const searchRAGRepositories = (
  repositories: RAGRepository[],
  query: string,
): RAGRepository[] => {
  if (!query.trim()) return repositories

  const searchTerm = query.toLowerCase()
  return repositories.filter(
    repository =>
      repository.name.toLowerCase().includes(searchTerm) ||
      repository.description?.toLowerCase().includes(searchTerm) ||
      repository.url.toLowerCase().includes(searchTerm),
  )
}

export const searchAvailableRAGDatabases = (
  databases: RAGDatabase[],
  query: string,
): RAGDatabase[] => {
  if (!query.trim()) return databases

  const searchTerm = query.toLowerCase()
  return databases.filter(
    database =>
      database.name.toLowerCase().includes(searchTerm) ||
      database.alias.toLowerCase().includes(searchTerm) ||
      database.description?.toLowerCase().includes(searchTerm),
  )
}
