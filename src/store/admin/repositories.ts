import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../../api/client.ts'
import {
  CreateRepositoryRequest,
  Repository,
  TestRepositoryConnectionRequest,
  UpdateRepositoryRequest,
} from '../../types/api/repository.ts'

interface RepositoriesState {
  // Data
  repositories: Repository[]

  // Loading states
  loading: boolean
  creating: boolean
  updating: boolean
  deleting: boolean
  testing: boolean

  // Error state
  error: string | null
}

export const useRepositoriesStore = create<RepositoriesState>()(
  subscribeWithSelector(
    (): RepositoriesState => ({
      // Initial state
      repositories: [],
      loading: false,
      creating: false,
      updating: false,
      deleting: false,
      testing: false,
      error: null,
    }),
  ),
)

// Repository actions
export const loadAllModelRepositories = async (): Promise<void> => {
  try {
    useRepositoriesStore.setState({ loading: true, error: null })

    const response = await ApiClient.Admin.listRepositories({
      page: 1,
      per_page: 50,
    })

    useRepositoriesStore.setState({
      repositories: response.repositories,
      loading: false,
    })
  } catch (error) {
    useRepositoriesStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to load repositories',
      loading: false,
    })
    throw error
  }
}

export const createNewModelRepository = async (
  data: CreateRepositoryRequest,
): Promise<Repository> => {
  try {
    useRepositoriesStore.setState({ creating: true, error: null })

    const repository = await ApiClient.Admin.createRepository(data)

    useRepositoriesStore.setState(state => ({
      repositories: [...state.repositories, repository],
      creating: false,
    }))

    return repository
  } catch (error) {
    useRepositoriesStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to create repository',
      creating: false,
    })
    throw error
  }
}

export const updateModelRepository = async (
  id: string,
  data: UpdateRepositoryRequest,
): Promise<Repository> => {
  try {
    useRepositoriesStore.setState({ updating: true, error: null })

    const repository = await ApiClient.Admin.updateRepository({
      repository_id: id,
      ...data,
    })

    useRepositoriesStore.setState(state => ({
      repositories: state.repositories.map(r => (r.id === id ? repository : r)),
      updating: false,
    }))

    return repository
  } catch (error) {
    useRepositoriesStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to update repository',
      updating: false,
    })
    throw error
  }
}

export const deleteModelRepository = async (id: string): Promise<void> => {
  try {
    useRepositoriesStore.setState({ deleting: true, error: null })

    await ApiClient.Admin.deleteRepository({ repository_id: id })

    useRepositoriesStore.setState(state => ({
      repositories: state.repositories.filter(r => r.id !== id),
      deleting: false,
    }))
  } catch (error) {
    useRepositoriesStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to delete repository',
      deleting: false,
    })
    throw error
  }
}

export const testModelRepositoryConnection = async (
  data: TestRepositoryConnectionRequest,
): Promise<{ success: boolean; message: string }> => {
  try {
    useRepositoriesStore.setState({ testing: true, error: null })

    const result = await ApiClient.Admin.testRepositoryConnection(data)

    useRepositoriesStore.setState({ testing: false })

    return result
  } catch (error) {
    useRepositoriesStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to test repository connection',
      testing: false,
    })
    throw error
  }
}

export const clearRepositoriesStoreError = (): void => {
  useRepositoriesStore.setState({ error: null })
}

export const findRepositoryById = (id: string): Repository | undefined => {
  return useRepositoriesStore.getState().repositories.find(r => r.id === id)
}

export const repositoryHasCredentials = (repository: Repository): boolean => {
  // If auth type is none, no credentials are needed
  if (repository.auth_type === 'none') {
    return true
  }

  // Check if auth_config exists
  if (!repository.auth_config) {
    return false
  }

  // Check credentials based on auth type
  switch (repository.auth_type) {
    case 'api_key':
      return !!(
        repository.auth_config.api_key && repository.auth_config.api_key.trim()
      )

    case 'basic_auth':
      return !!(
        repository.auth_config.username &&
        repository.auth_config.username.trim() &&
        repository.auth_config.password &&
        repository.auth_config.password.trim()
      )

    case 'bearer_token':
      return !!(
        repository.auth_config.token && repository.auth_config.token.trim()
      )

    default:
      return false
  }
}
