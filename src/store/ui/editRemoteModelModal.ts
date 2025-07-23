import { create } from 'zustand'

interface EditRemoteModelModalState {
  open: boolean
  loading: boolean
  modelId: string | null
}

export const useEditRemoteModelModalStore = create<EditRemoteModelModalState>(
  () => ({
    open: false,
    loading: false,
    modelId: null,
  }),
)

// Modal actions
export const openEditRemoteModelModal = (modelId: string) => {
  useEditRemoteModelModalStore.setState({
    open: true,
    modelId,
  })
}

export const closeEditRemoteModelModal = () => {
  useEditRemoteModelModalStore.setState({
    open: false,
    loading: false,
    modelId: null,
  })
}

export const setEditRemoteModelModalLoading = (loading: boolean) => {
  useEditRemoteModelModalStore.setState({
    loading,
  })
}
