import { create } from 'zustand'

interface EditLocalModelDrawerState {
  open: boolean
  loading: boolean
  modelId: string | null
}

export const useEditLocalModelDrawerStore = create<EditLocalModelDrawerState>(
  () => ({
    open: false,
    loading: false,
    modelId: null,
  }),
)

// Modal actions
export const openEditLocalModelDrawer = (modelId: string) => {
  useEditLocalModelDrawerStore.setState({
    open: true,
    modelId,
  })
}

export const closeEditLocalModelDrawer = () => {
  useEditLocalModelDrawerStore.setState({
    open: false,
    loading: false,
    modelId: null,
  })
}

export const setEditLocalModelDrawerLoading = (loading: boolean) => {
  useEditLocalModelDrawerStore.setState({
    loading,
  })
}
