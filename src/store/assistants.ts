import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../api/client'
import { Assistant } from '../types/api/assistant'

interface AssistantsState {
  // Data
  assistants: Assistant[]
  adminAssistants: Assistant[]

  // Loading states
  loading: boolean
  creating: boolean
  updating: boolean
  deleting: boolean

  // Error state
  error: string | null
}

export const useAssistantsStore = create<AssistantsState>()(
  subscribeWithSelector(
    (): AssistantsState => ({
      // Initial state
      assistants: [],
      adminAssistants: [],
      loading: false,
      creating: false,
      updating: false,
      deleting: false,
      error: null,
    }),
  ),
)

// Assistants actions
export const loadUserAssistants = async (): Promise<void> => {
  try {
    useAssistantsStore.setState({ loading: true, error: null })

    const response = await ApiClient.Assistant.list({
      page: 1,
      per_page: 50,
    })

    useAssistantsStore.setState({
      assistants: response.assistants,
      loading: false,
    })
  } catch (error) {
    useAssistantsStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to load assistants',
      loading: false,
    })
    throw error
  }
}

export const loadAdministratorAssistants = async (): Promise<void> => {
  try {
    useAssistantsStore.setState({ loading: true, error: null })

    const response = await ApiClient.Admin.listAssistants({
      page: 1,
      per_page: 50,
    })

    useAssistantsStore.setState({
      adminAssistants: response.assistants,
      loading: false,
    })
  } catch (error) {
    useAssistantsStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to load admin assistants',
      loading: false,
    })
    throw error
  }
}

export const createUserAssistant = async (
  data: Partial<Assistant>,
): Promise<Assistant> => {
  try {
    useAssistantsStore.setState({ creating: true, error: null })

    const assistant = await ApiClient.Assistant.create(data as any)

    useAssistantsStore.setState(state => ({
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
    useAssistantsStore.setState({
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
    useAssistantsStore.setState({ updating: true, error: null })

    const assistant = await ApiClient.Assistant.update({
      assistant_id: id,
      ...data,
    })

    useAssistantsStore.setState(state => ({
      assistants: data.is_default
        ? state.assistants.map(a =>
            a.id === id ? assistant : { ...a, is_default: false },
          )
        : state.assistants.map(a => (a.id === id ? assistant : a)),
      updating: false,
    }))

    return assistant
  } catch (error) {
    useAssistantsStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to update assistant',
      updating: false,
    })
    throw error
  }
}

export const deleteUserAssistant = async (id: string): Promise<void> => {
  try {
    useAssistantsStore.setState({ deleting: true, error: null })

    await ApiClient.Assistant.delete({ assistant_id: id })

    useAssistantsStore.setState(state => ({
      assistants: state.assistants.filter(a => a.id !== id),
      deleting: false,
    }))
  } catch (error) {
    useAssistantsStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to delete assistant',
      deleting: false,
    })
    throw error
  }
}

export const createAdministratorAssistant = async (
  data: Partial<Assistant>,
): Promise<Assistant> => {
  try {
    useAssistantsStore.setState({ creating: true, error: null })

    const assistant = await ApiClient.Admin.createAssistant(data as any)

    useAssistantsStore.setState(state => ({
      adminAssistants: [...state.adminAssistants, assistant],
      creating: false,
    }))

    return assistant
  } catch (error) {
    useAssistantsStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to create admin assistant',
      creating: false,
    })
    throw error
  }
}

export const updateAdministratorAssistant = async (
  id: string,
  data: Partial<Assistant>,
): Promise<Assistant> => {
  try {
    useAssistantsStore.setState({ updating: true, error: null })

    const assistant = await ApiClient.Admin.updateAssistant({
      assistant_id: id,
      ...data,
    })

    useAssistantsStore.setState(state => ({
      adminAssistants: state.adminAssistants.map(a =>
        a.id === id ? assistant : a,
      ),
      updating: false,
    }))

    return assistant
  } catch (error) {
    useAssistantsStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to update admin assistant',
      updating: false,
    })
    throw error
  }
}

export const deleteAdministratorAssistant = async (
  id: string,
): Promise<void> => {
  try {
    useAssistantsStore.setState({ deleting: true, error: null })

    await ApiClient.Admin.deleteAssistant({ assistant_id: id })

    useAssistantsStore.setState(state => ({
      adminAssistants: state.adminAssistants.filter(a => a.id !== id),
      deleting: false,
    }))
  } catch (error) {
    useAssistantsStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to delete admin assistant',
      deleting: false,
    })
    throw error
  }
}

export const clearAssistantsStoreError = (): void => {
  useAssistantsStore.setState({ error: null })
}
