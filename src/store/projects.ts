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

  // Actions
  loadProjects: () => Promise<void>
  loadProject: (id: string) => Promise<void>
  loadProjectDetails: (id: string) => Promise<void>
  createProject: (data: {
    name: string
    description: string
  }) => Promise<Project>
  updateProject: (
    id: string,
    data: { name?: string; description?: string },
  ) => Promise<Project>
  deleteProject: (id: string) => Promise<void>
  uploadDocument: (
    projectId: string,
    file: any,
  ) => Promise<UploadDocumentResponse>
  clearError: () => void
  reset: () => void
}

export const useProjectsStore = create<ProjectsState>()(
  subscribeWithSelector(set => ({
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

    loadProjects: async () => {
      try {
        set({ loading: true, error: null })

        const response = await ApiClient.Projects.list({
          page: 1,
          per_page: 50,
        })

        set({
          projects: response.projects,
          loading: false,
        })
      } catch (error) {
        set({
          error:
            error instanceof Error ? error.message : 'Failed to load projects',
          loading: false,
        })
        throw error
      }
    },

    loadProject: async (id: string) => {
      try {
        set({ loading: true, error: null })

        const response = await ApiClient.Projects.get({ project_id: id })

        set({
          currentProject: response.project,
          loading: false,
        })
      } catch (error) {
        set({
          error:
            error instanceof Error ? error.message : 'Failed to load project',
          loading: false,
        })
        throw error
      }
    },

    loadProjectDetails: async (id: string) => {
      try {
        set({ loading: true, error: null })

        const response = await ApiClient.Projects.get({ project_id: id })

        set({
          currentProject: response.project,
          documents: (response.project as any).documents || [],
          conversations: (response.project as any).conversations || [],
          loading: false,
        })
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to load project details',
          loading: false,
        })
        throw error
      }
    },

    createProject: async (data: { name: string; description: string }) => {
      try {
        set({ creating: true, error: null })

        const project = await ApiClient.Projects.create(data)

        set(state => ({
          projects: [...state.projects, project],
          creating: false,
        }))

        return project
      } catch (error) {
        set({
          error:
            error instanceof Error ? error.message : 'Failed to create project',
          creating: false,
        })
        throw error
      }
    },

    updateProject: async (
      id: string,
      data: { name?: string; description?: string },
    ) => {
      try {
        set({ updating: true, error: null })

        const project = await ApiClient.Projects.update({
          project_id: id,
          ...data,
        })

        set(state => ({
          projects: state.projects.map(p => (p.id === id ? project : p)),
          currentProject:
            state.currentProject?.id === id ? project : state.currentProject,
          updating: false,
        }))

        return project
      } catch (error) {
        set({
          error:
            error instanceof Error ? error.message : 'Failed to update project',
          updating: false,
        })
        throw error
      }
    },

    deleteProject: async (id: string) => {
      try {
        set({ deleting: true, error: null })

        await ApiClient.Projects.delete({ project_id: id })

        set(state => ({
          projects: state.projects.filter(p => p.id !== id),
          currentProject:
            state.currentProject?.id === id ? null : state.currentProject,
          deleting: false,
        }))
      } catch (error) {
        set({
          error:
            error instanceof Error ? error.message : 'Failed to delete project',
          deleting: false,
        })
        throw error
      }
    },

    uploadDocument: async (projectId: string, file: any) => {
      try {
        set({ uploading: true, error: null })

        const response = await ApiClient.Projects.uploadDocument({
          project_id: projectId,
          file: file,
        } as any)

        set(state => ({
          documents: [...state.documents, response.document],
          uploading: false,
        }))

        return response
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to upload document',
          uploading: false,
        })
        throw error
      }
    },

    clearError: () => {
      set({ error: null })
    },

    reset: () => {
      set({
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
    },
  })),
)
