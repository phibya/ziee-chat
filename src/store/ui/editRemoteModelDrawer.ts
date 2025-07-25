import { create } from 'zustand'

interface EditRemoteModelDrawerState {
  open: boolean
  loading: boolean
  modelId: string | null
}

export const useEditRemoteModelDrawerStore = create<EditRemoteModelDrawerState>(
  () => ({
    open: false,
    loading: false,
    modelId: null,
  }),
)

// Modal actions
export const openEditRemoteModelDrawer = (modelId: string) => {
  useEditRemoteModelDrawerStore.setState({
    open: true,
    modelId,
  })
}

export const closeEditRemoteModelDrawer = () => {
  useEditRemoteModelDrawerStore.setState({
    open: false,
    loading: false,
    modelId: null,
  })
}

export const setEditRemoteModelDrawerLoading = (loading: boolean) => {
  useEditRemoteModelDrawerStore.setState({
    loading,
  })
}
