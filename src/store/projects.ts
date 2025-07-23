import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../api/client'
import {
  Project,
  ProjectDocument,
  UploadDocumentResponse,
} from '../types/api/projects'

// Type alias for compatibility
type Document = ProjectDocument

interface Conversation {
  id: string
  title: string
  project_id: string
  last_message: string
  message_count: number
  created_at: string
  updated_at: string
}

interface ProjectsState {
  // Data
  projects: Project[]
  currentProject: Project | null
  documents: Document[]
  conversations: Conversation[]

  // Loading states
  loading: boolean
  creating: boolean
  updating: boolean
  deleting: boolean
  uploading: boolean

  // Error state
  error: string | null
}

export const useProjectsStore = create<ProjectsState>()(
  subscribeWithSelector(
    (): ProjectsState => ({
      // Initial state
      projects: [],
      currentProject: null,
      documents: [],
      conversations: [],
      loading: false,
      creating: false,
      updating: false,
      deleting: false,
      uploading: false,
      error: null,
    }),
  ),
)

// Project actions
export const loadAllUserProjects = async (): Promise<void> => {
  try {
    useProjectsStore.setState({ loading: true, error: null })

    const response = await ApiClient.Projects.list({
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

export const loadProjectById = async (id: string): Promise<void> => {
  try {
    useProjectsStore.setState({ loading: true, error: null })

    const response = await ApiClient.Projects.get({ project_id: id })

    useProjectsStore.setState({
      currentProject: response.project,
      loading: false,
    })
  } catch (error) {
    useProjectsStore.setState({
      error: error instanceof Error ? error.message : 'Failed to load project',
      loading: false,
    })
    throw error
  }
}

export const loadProjectWithDetails = async (id: string): Promise<void> => {
  try {
    useProjectsStore.setState({ loading: true, error: null })

    const response = await ApiClient.Projects.get({ project_id: id })

    useProjectsStore.setState({
      currentProject: response.project,
      documents: (response.project as any).documents || [],
      conversations: (response.project as any).conversations || [],
      loading: false,
    })
  } catch (error) {
    useProjectsStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to load project details',
      loading: false,
    })
    throw error
  }
}

export const createNewProject = async (data: {
  name: string
  description: string
}): Promise<Project> => {
  try {
    useProjectsStore.setState({ creating: true, error: null })

    const project = await ApiClient.Projects.create(data)

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

export const updateExistingProject = async (
  id: string,
  data: { name?: string; description?: string },
): Promise<Project> => {
  try {
    useProjectsStore.setState({ updating: true, error: null })

    const project = await ApiClient.Projects.update({
      project_id: id,
      ...data,
    })

    useProjectsStore.setState(state => ({
      projects: state.projects.map(p => (p.id === id ? project : p)),
      currentProject:
        state.currentProject?.id === id ? project : state.currentProject,
      updating: false,
    }))

    return project
  } catch (error) {
    useProjectsStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to update project',
      updating: false,
    })
    throw error
  }
}

export const deleteExistingProject = async (id: string): Promise<void> => {
  try {
    useProjectsStore.setState({ deleting: true, error: null })

    await ApiClient.Projects.delete({ project_id: id })

    useProjectsStore.setState(state => ({
      projects: state.projects.filter(p => p.id !== id),
      currentProject:
        state.currentProject?.id === id ? null : state.currentProject,
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

export const uploadDocumentToProject = async (
  projectId: string,
  file: any,
): Promise<UploadDocumentResponse> => {
  try {
    useProjectsStore.setState({ uploading: true, error: null })

    const response = await ApiClient.Projects.uploadDocument({
      project_id: projectId,
      file: file,
    } as any)

    useProjectsStore.setState(state => ({
      documents: [...state.documents, response.document],
      uploading: false,
    }))

    return response
  } catch (error) {
    useProjectsStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to upload document',
      uploading: false,
    })
    throw error
  }
}

export const clearProjectsStoreError = (): void => {
  useProjectsStore.setState({ error: null })
}

export const resetProjectsStore = (): void => {
  useProjectsStore.setState({
    projects: [],
    currentProject: null,
    documents: [],
    conversations: [],
    loading: false,
    creating: false,
    updating: false,
    deleting: false,
    uploading: false,
    error: null,
  })
}
