import { create } from 'zustand'

interface AddRemoteModelModalState {
  open: boolean
  loading: boolean
  providerId: string | null
  providerType: string | null
}

export const useAddRemoteModelModalStore = create<AddRemoteModelModalState>(
  () => ({
    open: false,
    loading: false,
    providerId: null,
    providerType: null,
  }),
)

// Modal actions
export const openAddRemoteModelModal = (
  providerId: string,
  providerType: string,
) => {
  useAddRemoteModelModalStore.setState({
    open: true,
    providerId,
    providerType,
  })
}

export const closeAddRemoteModelModal = () => {
  useAddRemoteModelModalStore.setState({
    open: false,
    loading: false,
    providerId: null,
    providerType: null,
  })
}

export const setAddRemoteModelModalLoading = (loading: boolean) => {
  useAddRemoteModelModalStore.setState({
    loading,
  })
}
