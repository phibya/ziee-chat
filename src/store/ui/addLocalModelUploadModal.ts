import { create } from 'zustand'

interface AddLocalModelUploadModalState {
  open: boolean
  loading: boolean
  providerId: string | null
}

export const useAddLocalModelUploadModalStore =
  create<AddLocalModelUploadModalState>(() => ({
    open: false,
    loading: false,
    providerId: null,
  }))

// Modal actions
export const openAddLocalModelUploadModal = (providerId: string) => {
  useAddLocalModelUploadModalStore.setState({
    open: true,
    providerId,
  })
}

export const closeAddLocalModelUploadModal = () => {
  useAddLocalModelUploadModalStore.setState({
    open: false,
    loading: false,
    providerId: null,
  })
}

export const setAddLocalModelUploadModalLoading = (loading: boolean) => {
  useAddLocalModelUploadModalStore.setState({
    loading,
  })
}
