import { create } from 'zustand'
import { RAGDatabase } from '../../types'

interface EditRAGDatabaseDrawerState {
  open: boolean
  loading: boolean
  database?: RAGDatabase
}

export const useEditRAGDatabaseDrawerStore = create<EditRAGDatabaseDrawerState>(
  () => ({
    open: false,
    loading: false,
    database: undefined,
  }),
)

// Modal actions
export const openEditRAGDatabaseDrawer = (database: RAGDatabase) => {
  useEditRAGDatabaseDrawerStore.setState({
    open: true,
    database,
  })
}

export const closeEditRAGDatabaseDrawer = () => {
  useEditRAGDatabaseDrawerStore.setState({
    open: false,
    loading: false,
    database: undefined,
  })
}

export const setEditRAGDatabaseDrawerLoading = (loading: boolean) => {
  useEditRAGDatabaseDrawerStore.setState({
    loading,
  })
}
