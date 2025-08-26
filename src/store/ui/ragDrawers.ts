import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import type { RAGInstance } from '../../types/api'

// RAG Instance Drawer Store
interface RAGInstanceDrawerState {
  open: boolean
  loading: boolean
  editingInstance: RAGInstance | null
}

export const useRAGInstanceDrawerStore = create<RAGInstanceDrawerState>()(
  subscribeWithSelector((_set, _get) => ({
    open: false,
    loading: false,
    editingInstance: null,
  })),
)

// RAG Instance Drawer Actions
export const openRAGInstanceDrawer = (instance?: RAGInstance) => {
  useRAGInstanceDrawerStore.setState({
    open: true,
    editingInstance: instance || null,
    loading: false,
  })
}

export const closeRAGInstanceDrawer = () => {
  useRAGInstanceDrawerStore.setState({
    open: false,
    editingInstance: null,
    loading: false,
  })
}

export const setRAGInstanceDrawerLoading = (loading: boolean) => {
  useRAGInstanceDrawerStore.setState({ loading })
}