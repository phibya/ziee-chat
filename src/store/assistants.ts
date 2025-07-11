import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../api/client'
import { Assistant } from '../types/api/assistant'

interface AssistantsState {
  // Data
  assistants: Assistant[]
  adminAssistants: Assistant[]

  // Loading states
  loading: boolean
  creating: boolean
  updating: boolean
  deleting: boolean

  // Error state
  error: string | null

  // Actions
  loadAssistants: () => Promise<void>
  loadAdminAssistants: () => Promise<void>
  createAssistant: (data: Partial<Assistant>) => Promise<Assistant>
  updateAssistant: (id: string, data: Partial<Assistant>) => Promise<Assistant>
  deleteAssistant: (id: string) => Promise<void>
  createAdminAssistant: (data: Partial<Assistant>) => Promise<Assistant>
  updateAdminAssistant: (
    id: string,
    data: Partial<Assistant>,
  ) => Promise<Assistant>
  deleteAdminAssistant: (id: string) => Promise<void>
  clearError: () => void
}

export const useAssistantsStore = create<AssistantsState>()(
  subscribeWithSelector(set => ({
    // Initial state
    assistants: [],
    adminAssistants: [],
    loading: false,
    creating: false,
    updating: false,
    deleting: false,
    error: null,

    loadAssistants: async () => {
      try {
        set({ loading: true, error: null })

        const response = await ApiClient.Assistant.list({
          page: 1,
          per_page: 50,
        })

        set({
          assistants: response.assistants,
          loading: false,
        })
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to load assistants',
          loading: false,
        })
        throw error
      }
    },

    loadAdminAssistants: async () => {
      try {
        set({ loading: true, error: null })

        const response = await ApiClient.Assistant.list({
          page: 1,
          per_page: 50,
        })

        set({
          adminAssistants: response.assistants,
          loading: false,
        })
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to load admin assistants',
          loading: false,
        })
        throw error
      }
    },

    createAssistant: async (data: Partial<Assistant>) => {
      try {
        set({ creating: true, error: null })

        const assistant = await ApiClient.Assistant.create(data as any)

        set(state => ({
          assistants: [...state.assistants, assistant],
          creating: false,
        }))

        return assistant
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to create assistant',
          creating: false,
        })
        throw error
      }
    },

    updateAssistant: async (id: string, data: Partial<Assistant>) => {
      try {
        set({ updating: true, error: null })

        const assistant = await ApiClient.Assistant.update({
          assistant_id: id,
          ...data,
        })

        set(state => ({
          assistants: state.assistants.map(a => (a.id === id ? assistant : a)),
          updating: false,
        }))

        return assistant
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to update assistant',
          updating: false,
        })
        throw error
      }
    },

    deleteAssistant: async (id: string) => {
      try {
        set({ deleting: true, error: null })

        await ApiClient.Assistant.delete({ assistant_id: id })

        set(state => ({
          assistants: state.assistants.filter(a => a.id !== id),
          deleting: false,
        }))
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to delete assistant',
          deleting: false,
        })
        throw error
      }
    },

    createAdminAssistant: async (data: Partial<Assistant>) => {
      try {
        set({ creating: true, error: null })

        const assistant = await ApiClient.Assistant.create(data as any)

        set(state => ({
          adminAssistants: [...state.adminAssistants, assistant],
          creating: false,
        }))

        return assistant
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to create admin assistant',
          creating: false,
        })
        throw error
      }
    },

    updateAdminAssistant: async (id: string, data: Partial<Assistant>) => {
      try {
        set({ updating: true, error: null })

        const assistant = await ApiClient.Assistant.update({
          assistant_id: id,
          ...data,
        })

        set(state => ({
          adminAssistants: state.adminAssistants.map(a =>
            a.id === id ? assistant : a,
          ),
          updating: false,
        }))

        return assistant
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to update admin assistant',
          updating: false,
        })
        throw error
      }
    },

    deleteAdminAssistant: async (id: string) => {
      try {
        set({ deleting: true, error: null })

        await ApiClient.Assistant.delete({ assistant_id: id })

        set(state => ({
          adminAssistants: state.adminAssistants.filter(a => a.id !== id),
          deleting: false,
        }))
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to delete admin assistant',
          deleting: false,
        })
        throw error
      }
    },

    clearError: () => {
      set({ error: null })
    },
  })),
)
