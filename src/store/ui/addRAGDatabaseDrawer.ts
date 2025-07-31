import { create } from 'zustand'

interface AddRAGDatabaseDrawerState {
  open: boolean
  loading: boolean
  providerId?: string
}

export const useAddRAGDatabaseDrawerStore = create<AddRAGDatabaseDrawerState>(
  () => ({
    open: false,
    loading: false,
    providerId: undefined,
  }),
)

// Modal actions
export const openAddRAGDatabaseDrawer = (providerId?: string) => {
  useAddRAGDatabaseDrawerStore.setState({
    open: true,
    providerId,
  })
}

export const closeAddRAGDatabaseDrawer = () => {
  useAddRAGDatabaseDrawerStore.setState({
    open: false,
    loading: false,
    providerId: undefined,
  })
}

export const setAddRAGDatabaseDrawerLoading = (loading: boolean) => {
  useAddRAGDatabaseDrawerStore.setState({
    loading,
  })
}
