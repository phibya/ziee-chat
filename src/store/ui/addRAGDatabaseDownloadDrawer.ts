import { create } from 'zustand'

interface AddRAGDatabaseDownloadDrawerState {
  open: boolean
  loading: boolean
  providerId?: string
}

export const useAddRAGDatabaseDownloadDrawerStore = create<AddRAGDatabaseDownloadDrawerState>(() => ({
  open: false,
  loading: false,
  providerId: undefined,
}))

// Modal actions
export const openAddRAGDatabaseDownloadDrawer = (providerId?: string) => {
  useAddRAGDatabaseDownloadDrawerStore.setState({
    open: true,
    providerId,
  })
}

export const closeAddRAGDatabaseDownloadDrawer = () => {
  useAddRAGDatabaseDownloadDrawerStore.setState({
    open: false,
    loading: false,
    providerId: undefined,
  })
}

export const setAddRAGDatabaseDownloadDrawerLoading = (loading: boolean) => {
  useAddRAGDatabaseDownloadDrawerStore.setState({
    loading,
  })
}