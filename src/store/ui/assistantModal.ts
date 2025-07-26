import { create } from 'zustand'
import { Assistant } from '../../types/api/assistant'

interface AssistantDrawerState {
  open: boolean
  loading: boolean
  editingAssistant: Assistant | null
  isAdmin: boolean
  isCloning: boolean
}

export const useAssistantDrawerStore = create<AssistantDrawerState>(() => ({
  open: false,
  loading: false,
  editingAssistant: null,
  isAdmin: false,
  isCloning: false,
}))

// Modal actions
export const openAssistantDrawer = (
  assistant?: Assistant,
  isAdmin = false,
  isCloning = false,
) => {
  useAssistantDrawerStore.setState({
    open: true,
    editingAssistant: assistant || null,
    isAdmin,
    isCloning,
  })
}

export const closeAssistantDrawer = () => {
  useAssistantDrawerStore.setState({
    open: false,
    loading: false,
    editingAssistant: null,
    isAdmin: false,
    isCloning: false,
  })
}

export const setAssistantDrawerLoading = (loading: boolean) => {
  useAssistantDrawerStore.setState({
    loading,
  })
}
