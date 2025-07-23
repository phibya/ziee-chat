import { create } from 'zustand'
import { Assistant } from '../../types/api/assistant'

interface ModalsUIState {
  // Assistant modal state
  assistantModalOpen: boolean
  assistantModalLoading: boolean
  editingAssistant: Assistant | null

  // Provider modal state
  addProviderModalOpen: boolean
  addProviderModalLoading: boolean
  editProviderModalOpen: boolean
  editProviderModalLoading: boolean
  editingProviderId: string | null

  // Model modal state
  addModelModalOpen: boolean
  addModelModalLoading: boolean
  editModelModalOpen: boolean
  editModelModalLoading: boolean
  editingModelId: string | null
}

export const useModalsUIStore = create<ModalsUIState>(() => ({
  // Initial state
  assistantModalOpen: false,
  assistantModalLoading: false,
  editingAssistant: null,

  addProviderModalOpen: false,
  addProviderModalLoading: false,
  editProviderModalOpen: false,
  editProviderModalLoading: false,
  editingProviderId: null,

  addModelModalOpen: false,
  addModelModalLoading: false,
  editModelModalOpen: false,
  editModelModalLoading: false,
  editingModelId: null,
}))

// Assistant modal actions
export const openAssistantModal = (assistant?: Assistant) => {
  useModalsUIStore.setState({
    assistantModalOpen: true,
    editingAssistant: assistant || null,
  })
}

export const closeAssistantModal = () => {
  useModalsUIStore.setState({
    assistantModalOpen: false,
    editingAssistant: null,
    assistantModalLoading: false,
  })
}

export const setAssistantModalLoading = (loading: boolean) => {
  useModalsUIStore.setState({
    assistantModalLoading: loading,
  })
}

// Provider modal actions
export const openAddProviderModal = () => {
  useModalsUIStore.setState({
    addProviderModalOpen: true,
  })
}

export const closeAddProviderModal = () => {
  useModalsUIStore.setState({
    addProviderModalOpen: false,
    addProviderModalLoading: false,
  })
}

export const setAddProviderModalLoading = (loading: boolean) => {
  useModalsUIStore.setState({
    addProviderModalLoading: loading,
  })
}

export const openEditProviderModal = (providerId: string) => {
  useModalsUIStore.setState({
    editProviderModalOpen: true,
    editingProviderId: providerId,
  })
}

export const closeEditProviderModal = () => {
  useModalsUIStore.setState({
    editProviderModalOpen: false,
    editingProviderId: null,
    editProviderModalLoading: false,
  })
}

export const setEditProviderModalLoading = (loading: boolean) => {
  useModalsUIStore.setState({
    editProviderModalLoading: loading,
  })
}

// Model modal actions
export const openAddModelModal = () => {
  useModalsUIStore.setState({
    addModelModalOpen: true,
  })
}

export const closeAddModelModal = () => {
  useModalsUIStore.setState({
    addModelModalOpen: false,
    addModelModalLoading: false,
  })
}

export const setAddModelModalLoading = (loading: boolean) => {
  useModalsUIStore.setState({
    addModelModalLoading: loading,
  })
}

export const openEditModelModal = (modelId: string) => {
  useModalsUIStore.setState({
    editModelModalOpen: true,
    editingModelId: modelId,
  })
}

export const closeEditModelModal = () => {
  useModalsUIStore.setState({
    editModelModalOpen: false,
    editingModelId: null,
    editModelModalLoading: false,
  })
}

export const setEditModelModalLoading = (loading: boolean) => {
  useModalsUIStore.setState({
    editModelModalLoading: loading,
  })
}

// Reset all modal states
export const resetModals = () => {
  useModalsUIStore.setState({
    assistantModalOpen: false,
    assistantModalLoading: false,
    editingAssistant: null,

    addProviderModalOpen: false,
    addProviderModalLoading: false,
    editProviderModalOpen: false,
    editProviderModalLoading: false,
    editingProviderId: null,

    addModelModalOpen: false,
    addModelModalLoading: false,
    editModelModalOpen: false,
    editModelModalLoading: false,
    editingModelId: null,
  })
}
