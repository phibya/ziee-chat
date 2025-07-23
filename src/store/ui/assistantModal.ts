import { create } from 'zustand'
import { Assistant } from '../../types/api/assistant'

interface AssistantModalState {
  open: boolean
  loading: boolean
  editingAssistant: Assistant | null
}

export const useAssistantModalStore = create<AssistantModalState>(() => ({
  open: false,
  loading: false,
  editingAssistant: null,
}))

// Modal actions
export const openAssistantModal = (assistant?: Assistant) => {
  useAssistantModalStore.setState({
    open: true,
    editingAssistant: assistant || null,
  })
}

export const closeAssistantModal = () => {
  useAssistantModalStore.setState({
    open: false,
    loading: false,
    editingAssistant: null,
  })
}

export const setAssistantModalLoading = (loading: boolean) => {
  useAssistantModalStore.setState({
    loading,
  })
}
