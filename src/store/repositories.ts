import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../api/client'
import {
  CreateRepositoryRequest,
  Repository,
  TestRepositoryConnectionRequest,
  UpdateRepositoryRequest,
} from '../types/api/repository'

interface RepositoriesState {
  // Data
  repositories: Repository[]

  // Loading states
  loading: boolean
  creating: boolean
  updating: boolean
  deleting: boolean
  testing: boolean

  // Error state
  error: string | null

  // Actions
  loadRepositories: () => Promise<void>
  createRepository: (data: CreateRepositoryRequest) => Promise<Repository>
  updateRepository: (
    id: string,
    data: UpdateRepositoryRequest,
  ) => Promise<Repository>
  deleteRepository: (id: string) => Promise<void>
  testConnection: (
    data: TestRepositoryConnectionRequest,
  ) => Promise<{ success: boolean; message: string }>

  // Utility actions
  clearError: () => void
  getRepositoryById: (id: string) => Repository | undefined
}

export const useRepositoriesStore = create<RepositoriesState>()(
  subscribeWithSelector((set, get) => ({
    // Initial state
    repositories: [],
    loading: false,
    creating: false,
    updating: false,
    deleting: false,
    testing: false,
    error: null,

    loadRepositories: async () => {
      try {
        set({ loading: true, error: null })

        const response = await ApiClient.Admin.listRepositories({
          page: 1,
          per_page: 50,
        })

        set({
          repositories: response.repositories,
          loading: false,
        })
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to load repositories',
          loading: false,
        })
        throw error
      }
    },

    createRepository: async (data: CreateRepositoryRequest) => {
      try {
        set({ creating: true, error: null })

        const repository = await ApiClient.Admin.createRepository(data)

        set(state => ({
          repositories: [...state.repositories, repository],
          creating: false,
        }))

        return repository
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to create repository',
          creating: false,
        })
        throw error
      }
    },

    updateRepository: async (id: string, data: UpdateRepositoryRequest) => {
      try {
        set({ updating: true, error: null })

        const repository = await ApiClient.Admin.updateRepository({
          repository_id: id,
          ...data,
        })

        set(state => ({
          repositories: state.repositories.map(r =>
            r.id === id ? repository : r,
          ),
          updating: false,
        }))

        return repository
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to update repository',
          updating: false,
        })
        throw error
      }
    },

    deleteRepository: async (id: string) => {
      try {
        set({ deleting: true, error: null })

        await ApiClient.Admin.deleteRepository({ repository_id: id })

        set(state => ({
          repositories: state.repositories.filter(r => r.id !== id),
          deleting: false,
        }))
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to delete repository',
          deleting: false,
        })
        throw error
      }
    },

    testConnection: async (data: TestRepositoryConnectionRequest) => {
      try {
        set({ testing: true, error: null })

        const result = await ApiClient.Admin.testRepositoryConnection(data)

        set({ testing: false })

        return result
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to test repository connection',
          testing: false,
        })
        throw error
      }
    },

    clearError: () => {
      set({ error: null })
    },

    getRepositoryById: (id: string) => {
      return get().repositories.find(r => r.id === id)
    },
  })),
)
