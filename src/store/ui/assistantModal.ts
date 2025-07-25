import { create } from 'zustand'
import { Assistant } from '../../types/api/assistant'

interface AssistantDrawerState {
  open: boolean
  loading: boolean
  editingAssistant: Assistant | null
}

export const useAssistantDrawerStore = create<AssistantDrawerState>(() => ({
  open: false,
  loading: false,
  editingAssistant: null,
}))

// Modal actions
export const openAssistantDrawer = (assistant?: Assistant) => {
  useAssistantDrawerStore.setState({
    open: true,
    editingAssistant: assistant || null,
  })
}

export const closeAssistantDrawer = () => {
  useAssistantDrawerStore.setState({
    open: false,
    loading: false,
    editingAssistant: null,
  })
}

export const setAssistantDrawerLoading = (loading: boolean) => {
  useAssistantDrawerStore.setState({
    loading,
  })
}
