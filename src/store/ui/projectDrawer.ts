import { create } from 'zustand'
import { Project } from '../../types/api/projects'

interface ProjectDrawerState {
  open: boolean
  loading: boolean
  editingProject: Project | null
}

export const useProjectDrawerStore = create<ProjectDrawerState>(() => ({
  open: false,
  loading: false,
  editingProject: null,
}))

// Drawer actions
export const openProjectDrawer = (project?: Project) => {
  useProjectDrawerStore.setState({
    open: true,
    editingProject: project || null,
  })
}

export const closeProjectDrawer = () => {
  useProjectDrawerStore.setState({
    open: false,
    loading: false,
    editingProject: null,
  })
}

export const setProjectDrawerLoading = (loading: boolean) => {
  useProjectDrawerStore.setState({
    loading,
  })
}
