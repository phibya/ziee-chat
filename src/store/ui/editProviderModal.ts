import { create } from 'zustand'

interface EditProviderModalState {
  open: boolean
  loading: boolean
  providerId: string | null
}

export const useEditProviderModalStore = create<EditProviderModalState>(() => ({
  open: false,
  loading: false,
  providerId: null,
}))

// Modal actions
export const openEditProviderModal = (providerId: string) => {
  useEditProviderModalStore.setState({
    open: true,
    providerId,
  })
}

export const closeEditProviderModal = () => {
  useEditProviderModalStore.setState({
    open: false,
    loading: false,
    providerId: null,
  })
}

export const setEditProviderModalLoading = (loading: boolean) => {
  useEditProviderModalStore.setState({
    loading,
  })
}
