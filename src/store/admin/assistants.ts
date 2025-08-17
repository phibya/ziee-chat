import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../../api/client'
import { Assistant, CreateAssistantRequest } from '../../types/api/assistant'

interface AdminAssistantsState {
  // Data
  assistants: Assistant[]
  total: number
  currentPage: number
  pageSize: number

  // Loading states
  loading: boolean
  creating: boolean
  updating: boolean
  deleting: boolean

  // Error state
  error: string | null
}

export const useAdminAssistantsStore = create<AdminAssistantsState>()(
  subscribeWithSelector(
    (): AdminAssistantsState => ({
      // Initial state
      assistants: [],
      total: 0,
      currentPage: 1,
      pageSize: 10,
      loading: false,
      creating: false,
      updating: false,
      deleting: false,
      error: null,
    }),
  ),
)

// Admin assistants actions
export const loadAdministratorAssistants = async (page?: number, pageSize?: number): Promise<void> => {
  try {
    const currentState = useAdminAssistantsStore.getState()
    const requestPage = page || currentState.currentPage
    const requestPageSize = pageSize || currentState.pageSize

    useAdminAssistantsStore.setState({ loading: true, error: null })

    const response = await ApiClient.Admin.listAssistants({
      page: requestPage,
      per_page: requestPageSize,
    })

    useAdminAssistantsStore.setState({
      assistants: response.assistants,
      total: response.total,
      currentPage: response.page,
      pageSize: response.per_page,
      loading: false,
    })
  } catch (error) {
    useAdminAssistantsStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to load admin assistants',
      loading: false,
    })
    throw error
  }
}

export const createSystemAdminAssistant = async (
  data: CreateAssistantRequest,
): Promise<Assistant> => {
  try {
    useAdminAssistantsStore.setState({ creating: true, error: null })

    const assistant = await ApiClient.Admin.createAssistant(data)

    useAdminAssistantsStore.setState(state => ({
      assistants: data.is_default
        ? [
            ...state.assistants.map(a => ({ ...a, is_default: false })),
            assistant,
          ]
        : [...state.assistants, assistant],
      creating: false,
    }))

    return assistant
  } catch (error) {
    useAdminAssistantsStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to create admin assistant',
      creating: false,
    })
    throw error
  }
}

export const updateSystemAdminAssistant = async (
  id: string,
  data: Partial<Assistant>,
): Promise<Assistant> => {
  try {
    useAdminAssistantsStore.setState({ updating: true, error: null })

    const assistant = await ApiClient.Admin.updateAssistant({
      assistant_id: id,
      ...data,
    })

    useAdminAssistantsStore.setState(state => ({
      assistants: data.is_default
        ? state.assistants.map(a =>
            a.id === id ? assistant : { ...a, is_default: false },
          )
        : state.assistants.map(a => (a.id === id ? assistant : a)),
      updating: false,
    }))

    return assistant
  } catch (error) {
    useAdminAssistantsStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to update admin assistant',
      updating: false,
    })
    throw error
  }
}

export const deleteSystemAdminAssistant = async (id: string): Promise<void> => {
  try {
    useAdminAssistantsStore.setState({ deleting: true, error: null })

    await ApiClient.Admin.deleteAssistant({ assistant_id: id })

    useAdminAssistantsStore.setState(state => ({
      assistants: state.assistants.filter(a => a.id !== id),
      deleting: false,
    }))
  } catch (error) {
    useAdminAssistantsStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to delete admin assistant',
      deleting: false,
    })
    throw error
  }
}

export const clearAdminAssistantsStoreError = (): void => {
  useAdminAssistantsStore.setState({ error: null })
}

// Legacy compatibility
export const loadSystemAdminAssistants = loadAdministratorAssistants
