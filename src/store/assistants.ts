import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../api/client'
import { Assistant } from '../types/api/assistant'

interface UserAssistantsState {
  // Data
  assistants: Assistant[]

  // Loading states
  loading: boolean
  creating: boolean
  updating: boolean
  deleting: boolean

  // Error state
  error: string | null
}

export const useUserAssistantsStore = create<UserAssistantsState>()(
  subscribeWithSelector(
    (): UserAssistantsState => ({
      // Initial state
      assistants: [],
      loading: false,
      creating: false,
      updating: false,
      deleting: false,
      error: null,
    }),
  ),
)

// User assistants actions
export const loadUserAssistants = async (): Promise<void> => {
  try {
    useUserAssistantsStore.setState({ loading: true, error: null })

    const response = await ApiClient.Assistant.list({
      page: 1,
      per_page: 50,
    })

    useUserAssistantsStore.setState({
      assistants: response.assistants,
      loading: false,
    })
  } catch (error) {
    useUserAssistantsStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to load assistants',
      loading: false,
    })
    throw error
  }
}

export const createUserAssistant = async (
  data: Partial<Assistant>,
): Promise<Assistant> => {
  try {
    useUserAssistantsStore.setState({ creating: true, error: null })

    const assistant = await ApiClient.Assistant.create(data as any)

    useUserAssistantsStore.setState(state => ({
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
    useUserAssistantsStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to create assistant',
      creating: false,
    })
    throw error
  }
}

export const updateUserAssistant = async (
  id: string,
  data: Partial<Assistant>,
): Promise<Assistant> => {
  try {
    useUserAssistantsStore.setState({ updating: true, error: null })

    const assistant = await ApiClient.Assistant.update({
      assistant_id: id,
      ...data,
    })

    useUserAssistantsStore.setState(state => ({
      assistants: data.is_default
        ? state.assistants.map(a =>
            a.id === id ? assistant : { ...a, is_default: false },
          )
        : state.assistants.map(a => (a.id === id ? assistant : a)),
      updating: false,
    }))

    return assistant
  } catch (error) {
    useUserAssistantsStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to update assistant',
      updating: false,
    })
    throw error
  }
}

export const deleteUserAssistant = async (id: string): Promise<void> => {
  try {
    useUserAssistantsStore.setState({ deleting: true, error: null })

    await ApiClient.Assistant.delete({ assistant_id: id })

    useUserAssistantsStore.setState(state => ({
      assistants: state.assistants.filter(a => a.id !== id),
      deleting: false,
    }))
  } catch (error) {
    useUserAssistantsStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to delete assistant',
      deleting: false,
    })
    throw error
  }
}


export const clearUserAssistantsStoreError = (): void => {
  useUserAssistantsStore.setState({ error: null })
}

// Legacy compatibility
export const useAssistantsStore = useUserAssistantsStore
export const clearAssistantsStoreError = clearUserAssistantsStoreError
