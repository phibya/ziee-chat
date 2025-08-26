import { create } from 'zustand'

// Add RAG Provider Drawer Store
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

export const openAddRAGProviderDrawer = (): void => {
  useAddRAGProviderDrawerStore.setState({ open: true })
}

export const closeAddRAGProviderDrawer = (): void => {
  useAddRAGProviderDrawerStore.setState({ open: false, loading: false })
}

export const setAddRAGProviderDrawerLoading = (loading: boolean): void => {
  useAddRAGProviderDrawerStore.setState({ loading })
}

// Edit RAG Provider Drawer Store
interface EditRAGProviderDrawerState {
  open: boolean
  loading: boolean
  providerId: string | null
}

export const useEditRAGProviderDrawerStore = create<EditRAGProviderDrawerState>(
  () => ({
    open: false,
    loading: false,
    providerId: null,
  }),
)

export const openEditRAGProviderDrawer = (providerId: string): void => {
  useEditRAGProviderDrawerStore.setState({ open: true, providerId })
}

export const closeEditRAGProviderDrawer = (): void => {
  useEditRAGProviderDrawerStore.setState({
    open: false,
    loading: false,
    providerId: null,
  })
}

export const setEditRAGProviderDrawerLoading = (loading: boolean): void => {
  useEditRAGProviderDrawerStore.setState({ loading })
}

// Add System Instance Drawer Store
interface AddSystemInstanceDrawerState {
  open: boolean
  loading: boolean
  providerId: string | null
}

export const useAddSystemInstanceDrawerStore =
  create<AddSystemInstanceDrawerState>(() => ({
    open: false,
    loading: false,
    providerId: null,
  }))

export const openAddSystemInstanceDrawer = (providerId: string): void => {
  useAddSystemInstanceDrawerStore.setState({ open: true, providerId })
}

export const closeAddSystemInstanceDrawer = (): void => {
  useAddSystemInstanceDrawerStore.setState({
    open: false,
    loading: false,
    providerId: null,
  })
}

export const setAddSystemInstanceDrawerLoading = (loading: boolean): void => {
  useAddSystemInstanceDrawerStore.setState({ loading })
}

// Edit System Instance Drawer Store
interface EditSystemInstanceDrawerState {
  open: boolean
  loading: boolean
  instanceId: string | null
}

export const useEditSystemInstanceDrawerStore =
  create<EditSystemInstanceDrawerState>(() => ({
    open: false,
    loading: false,
    instanceId: null,
  }))

export const openEditSystemInstanceDrawer = (instanceId: string): void => {
  useEditSystemInstanceDrawerStore.setState({ open: true, instanceId })
}

export const closeEditSystemInstanceDrawer = (): void => {
  useEditSystemInstanceDrawerStore.setState({
    open: false,
    loading: false,
    instanceId: null,
  })
}

export const setEditSystemInstanceDrawerLoading = (loading: boolean): void => {
  useEditSystemInstanceDrawerStore.setState({ loading })
}
