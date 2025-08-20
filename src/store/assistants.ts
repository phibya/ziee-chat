import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { immer } from 'zustand/middleware/immer'
import { enableMapSet } from 'immer'
import { ApiClient } from '../api/client'
import { Assistant } from '../types'

// Enable Map and Set support in Immer
enableMapSet()

interface UserAssistantsState {
  // Data
  assistants: Map<string, Assistant>
  isInitialized: boolean

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
    immer(
      (): UserAssistantsState => ({
        // Initial state
        assistants: new Map<string, Assistant>(),
        isInitialized: false,
        loading: false,
        creating: false,
        updating: false,
        deleting: false,
        error: null,
      }),
    ),
  ),
)

// User assistants actions
export const loadUserAssistants = async (): Promise<void> => {
  const state = useUserAssistantsStore.getState()
  if (state.isInitialized || state.loading) {
    return
  }
  try {
    useUserAssistantsStore.setState({ loading: true, error: null })

    const response = await ApiClient.Assistants.listAssistants({
      page: 1,
      per_page: 50,
    })

    useUserAssistantsStore.setState({
      assistants: new Map(
        response.assistants.map(assistant => [assistant.id, assistant]),
      ),
      isInitialized: true,
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

    const assistant = await ApiClient.Assistants.createAssistant(data as any)

    useUserAssistantsStore.setState(state => {
      if (data.is_default) {
        // Set all other assistants' is_default to false
        state.assistants.forEach(a => {
          a.is_default = false
        })
      }
      // Add the new assistant
      state.assistants.set(assistant.id, assistant)
      state.creating = false
    })

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

    const assistant = await ApiClient.Assistants.updateAssistant({
      assistant_id: id,
      ...data,
    })

    useUserAssistantsStore.setState(state => {
      if (data.is_default) {
        // Set all other assistants' is_default to false
        state.assistants.forEach((a, assistantId) => {
          if (assistantId !== id) {
            a.is_default = false
          }
        })
      }
      // Update the assistant
      state.assistants.set(id, assistant)
      state.updating = false
    })

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

    await ApiClient.Assistants.deleteAssistant({ assistant_id: id })

    useUserAssistantsStore.setState(state => {
      state.assistants.delete(id)
      state.deleting = false
    })
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
