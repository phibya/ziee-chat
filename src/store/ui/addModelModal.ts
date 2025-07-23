import { create } from 'zustand'

interface AddModelModalState {
  open: boolean
  loading: boolean
}

export const useAddModelModalStore = create<AddModelModalState>(() => ({
  open: false,
  loading: false,
}))

// Modal actions
export const openAddModelModal = () => {
  useAddModelModalStore.setState({
    open: true,
  })
}

export const closeAddModelModal = () => {
  useAddModelModalStore.setState({
    open: false,
    loading: false,
  })
}

export const setAddModelModalLoading = (loading: boolean) => {
  useAddModelModalStore.setState({
    loading,
  })
}
