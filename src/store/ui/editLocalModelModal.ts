import { create } from 'zustand'

interface EditLocalModelModalState {
  open: boolean
  loading: boolean
  modelId: string | null
}

export const useEditLocalModelModalStore = create<EditLocalModelModalState>(
  () => ({
    open: false,
    loading: false,
    modelId: null,
  }),
)

// Modal actions
export const openEditLocalModelModal = (modelId: string) => {
  useEditLocalModelModalStore.setState({
    open: true,
    modelId,
  })
}

export const closeEditLocalModelModal = () => {
  useEditLocalModelModalStore.setState({
    open: false,
    loading: false,
    modelId: null,
  })
}

export const setEditLocalModelModalLoading = (loading: boolean) => {
  useEditLocalModelModalStore.setState({
    loading,
  })
}
