import { create } from 'zustand'

interface AddRAGProviderDrawerState {
  open: boolean
  loading: boolean
}

export const useAddRAGProviderDrawerStore = create<AddRAGProviderDrawerState>(
  () => ({
    open: false,
    loading: false,
  }),
)

// Modal actions
export const openAddRAGProviderDrawer = () => {
  useAddRAGProviderDrawerStore.setState({
    open: true,
  })
}

export const closeAddRAGProviderDrawer = () => {
  useAddRAGProviderDrawerStore.setState({
    open: false,
    loading: false,
  })
}

export const setAddRAGProviderDrawerLoading = (loading: boolean) => {
  useAddRAGProviderDrawerStore.setState({
    loading,
  })
}
