import { create } from 'zustand'

interface AddModelDrawerState {
  open: boolean
  loading: boolean
}

export const useAddModelDrawerStore = create<AddModelDrawerState>(() => ({
  open: false,
  loading: false,
}))

// Modal actions
export const openAddModelDrawer = () => {
  useAddModelDrawerStore.setState({
    open: true,
  })
}

export const closeAddModelDrawer = () => {
  useAddModelDrawerStore.setState({
    open: false,
    loading: false,
  })
}

export const setAddModelDrawerLoading = (loading: boolean) => {
  useAddModelDrawerStore.setState({
    loading,
  })
}
