import { create } from 'zustand'

interface AddProviderModalState {
  open: boolean
  loading: boolean
}

export const useAddProviderModalStore = create<AddProviderModalState>(() => ({
  open: false,
  loading: false,
}))

// Modal actions
export const openAddProviderModal = () => {
  useAddProviderModalStore.setState({
    open: true,
  })
}

export const closeAddProviderModal = () => {
  useAddProviderModalStore.setState({
    open: false,
    loading: false,
  })
}

export const setAddProviderModalLoading = (loading: boolean) => {
  useAddProviderModalStore.setState({
    loading,
  })
}
