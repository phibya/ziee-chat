import { create } from 'zustand'

interface AddLocalModelDownloadModalState {
  open: boolean
  loading: boolean
  providerId: string | null
}

export const useAddLocalModelDownloadModalStore =
  create<AddLocalModelDownloadModalState>(() => ({
    open: false,
    loading: false,
    providerId: null,
  }))

// Modal actions
export const openAddLocalModelDownloadModal = (providerId: string) => {
  useAddLocalModelDownloadModalStore.setState({
    open: true,
    providerId,
  })
}

export const closeAddLocalModelDownloadModal = () => {
  useAddLocalModelDownloadModalStore.setState({
    open: false,
    loading: false,
    providerId: null,
  })
}

export const setAddLocalModelDownloadModalLoading = (loading: boolean) => {
  useAddLocalModelDownloadModalStore.setState({
    loading,
  })
}
