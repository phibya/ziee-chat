import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../api/client'
import { Project } from '../types'

interface ProjectsState {
  // Data
  projects: Project[]

  // Loading states
  loading: boolean
  creating: boolean
  deleting: boolean

  // Error state
  error: string | null
}

export const useProjectsStore = create<ProjectsState>()(
  subscribeWithSelector(
    (): ProjectsState => ({
      // Initial state
      projects: [],
      loading: false,
      creating: false,
      deleting: false,
      error: null,
    }),
  ),
)

// Project list actions
export const loadAllUserProjects = async (): Promise<void> => {
  try {
    useProjectsStore.setState({ loading: true, error: null })

    const response = await ApiClient.Projects.listProjects({
      page: 1,
      per_page: 50,
    })

    useProjectsStore.setState({
      projects: response.projects,
      loading: false,
    })
  } catch (error) {
    useProjectsStore.setState({
      error: error instanceof Error ? error.message : 'Failed to load projects',
      loading: false,
    })
    throw error
  }
}

export const createNewProject = async (data: {
  name: string
  description: string
  instruction?: string
}): Promise<Project> => {
  try {
    useProjectsStore.setState({ creating: true, error: null })

    const project = await ApiClient.Projects.createProject(data)

    useProjectsStore.setState(state => ({
      projects: [...state.projects, project],
      creating: false,
    }))

    return project
  } catch (error) {
    useProjectsStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to create project',
      creating: false,
    })
    throw error
  }
}

export const updateProjectInList = async (
  id: string,
  data: { name?: string; description?: string; instruction?: string },
): Promise<Project> => {
  try {
    const project = await ApiClient.Projects.updateProject({
      project_id: id,
      ...data,
    })

    useProjectsStore.setState(state => ({
      projects: state.projects.map(p => (p.id === id ? project : p)),
    }))

    return project
  } catch (error) {
    useProjectsStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to update project',
    })
    throw error
  }
}

export const deleteExistingProject = async (id: string): Promise<void> => {
  try {
    useProjectsStore.setState({ deleting: true, error: null })

    await ApiClient.Projects.deleteProject({ project_id: id })

    useProjectsStore.setState(state => ({
      projects: state.projects.filter(p => p.id !== id),
      deleting: false,
    }))
  } catch (error) {
    useProjectsStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to delete project',
      deleting: false,
    })
    throw error
  }
}

// Utility actions
export const clearProjectsStoreError = (): void => {
  useProjectsStore.setState({ error: null })
}

export const resetProjectsStore = (): void => {
  useProjectsStore.setState({
    projects: [],
    loading: false,
    creating: false,
    deleting: false,
    error: null,
  })
}
