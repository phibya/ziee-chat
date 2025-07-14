import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../api/client'
import {
  CreateModelProviderRequest,
  ModelProvider,
  ModelProviderModel,
} from '../types/api/modelProvider'

// Type alias for compatibility
type Model = ModelProviderModel

interface ModelProvidersState {
  // Data
  providers: ModelProvider[]

  // Loading states
  loading: boolean
  creating: boolean
  updating: boolean
  deleting: boolean
  testingProxy: boolean

  // Error state
  error: string | null

  // Actions
  loadProviders: () => Promise<void>
  createProvider: (data: CreateModelProviderRequest) => Promise<ModelProvider>
  updateProvider: (
    id: string,
    data: Partial<ModelProvider>,
  ) => Promise<ModelProvider>
  deleteProvider: (id: string) => Promise<void>
  cloneProvider: (id: string, name: string) => Promise<ModelProvider>

  // Model actions
  addModel: (providerId: string, data: Partial<Model>) => Promise<Model>
  updateModel: (modelId: string, data: Partial<Model>) => Promise<Model>
  deleteModel: (modelId: string) => Promise<void>
  startModel: (modelId: string) => Promise<void> // For Candle
  stopModel: (modelId: string) => Promise<void> // For Candle

  // Proxy actions
  testProxy: (providerId: string) => Promise<boolean>

  // Utility actions
  clearError: () => void
  getProviderById: (id: string) => ModelProvider | undefined
  getModelById: (id: string) => Model | undefined
}

export const useModelProvidersStore = create<ModelProvidersState>()(
  subscribeWithSelector((set, get) => ({
    // Initial state
    providers: [],
    loading: false,
    creating: false,
    updating: false,
    deleting: false,
    testingProxy: false,
    error: null,

    loadProviders: async () => {
      try {
        set({ loading: true, error: null })

        const response = await ApiClient.ModelProviders.list({
          page: 1,
          per_page: 50,
        })

        set({
          providers: response.providers,
          loading: false,
        })
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to load model providers',
          loading: false,
        })
        throw error
      }
    },

    createProvider: async (data: CreateModelProviderRequest) => {
      try {
        set({ creating: true, error: null })

        const provider = await ApiClient.ModelProviders.create(data)

        set(state => ({
          providers: [...state.providers, provider],
          creating: false,
        }))

        return provider
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to create provider',
          creating: false,
        })
        throw error
      }
    },

    updateProvider: async (id: string, data: Partial<ModelProvider>) => {
      try {
        set({ updating: true, error: null })

        const provider = await ApiClient.ModelProviders.update({
          provider_id: id,
          ...data,
        })

        set(state => ({
          providers: state.providers.map(p => (p.id === id ? provider : p)),
          updating: false,
        }))

        return provider
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to update provider',
          updating: false,
        })
        throw error
      }
    },

    deleteProvider: async (id: string) => {
      try {
        set({ deleting: true, error: null })

        await ApiClient.ModelProviders.delete({ provider_id: id })

        set(state => ({
          providers: state.providers.filter(p => p.id !== id),
          deleting: false,
        }))
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to delete provider',
          deleting: false,
        })
        throw error
      }
    },

    cloneProvider: async (id: string, name: string) => {
      try {
        set({ creating: true, error: null })

        const provider = await ApiClient.ModelProviders.clone({
          provider_id: id,
          name: name,
        } as any)

        set(state => ({
          providers: [...state.providers, provider],
          creating: false,
        }))

        return provider
      } catch (error) {
        set({
          error:
            error instanceof Error ? error.message : 'Failed to clone provider',
          creating: false,
        })
        throw error
      }
    },

    addModel: async (providerId: string, data: Partial<Model>) => {
      try {
        set({ creating: true, error: null })

        const model = await ApiClient.ModelProviders.addModel({
          provider_id: providerId,
          ...data,
        } as any)

        set(state => ({
          providers: state.providers.map(p =>
            p.id === providerId ? { ...p, models: [...p.models, model] } : p,
          ),
          creating: false,
        }))

        return model
      } catch (error) {
        set({
          error: error instanceof Error ? error.message : 'Failed to add model',
          creating: false,
        })
        throw error
      }
    },

    updateModel: async (modelId: string, data: Partial<Model>) => {
      try {
        set({ updating: true, error: null })

        const model = await ApiClient.Models.update({
          model_id: modelId,
          ...data,
        })

        set(state => ({
          providers: state.providers.map(p => ({
            ...p,
            models: p.models.map(m => (m.id === modelId ? model : m)),
          })),
          updating: false,
        }))

        return model
      } catch (error) {
        set({
          error:
            error instanceof Error ? error.message : 'Failed to update model',
          updating: false,
        })
        throw error
      }
    },

    deleteModel: async (modelId: string) => {
      try {
        set({ deleting: true, error: null })

        await ApiClient.Models.delete({ model_id: modelId })

        set(state => ({
          providers: state.providers.map(p => ({
            ...p,
            models: p.models.filter(m => m.id !== modelId),
          })),
          deleting: false,
        }))
      } catch (error) {
        set({
          error:
            error instanceof Error ? error.message : 'Failed to delete model',
          deleting: false,
        })
        throw error
      }
    },

    startModel: async (modelId: string) => {
      try {
        set({ updating: true, error: null })

        await (ApiClient.Models as any).start({ model_id: modelId })

        // Update model status to starting
        set(state => ({
          providers: state.providers.map(p => ({
            ...p,
            models: p.models.map(m =>
              m.id === modelId ? { ...m, status: 'starting' as const } : m,
            ),
          })),
          updating: false,
        }))
      } catch (error) {
        set({
          error:
            error instanceof Error ? error.message : 'Failed to start model',
          updating: false,
        })
        throw error
      }
    },

    stopModel: async (modelId: string) => {
      try {
        set({ updating: true, error: null })

        await (ApiClient.Models as any).stop({ model_id: modelId })

        // Update model status to stopping
        set(state => ({
          providers: state.providers.map(p => ({
            ...p,
            models: p.models.map(m =>
              m.id === modelId ? { ...m, status: 'stopping' as const } : m,
            ),
          })),
          updating: false,
        }))
      } catch (error) {
        set({
          error:
            error instanceof Error ? error.message : 'Failed to stop model',
          updating: false,
        })
        throw error
      }
    },

    testProxy: async (providerId: string) => {
      try {
        set({ testingProxy: true, error: null })

        const result = await ApiClient.ModelProviders.testProxy({
          provider_id: providerId,
        } as any)

        set({ testingProxy: false })

        return result.success
      } catch (error) {
        set({
          error:
            error instanceof Error ? error.message : 'Failed to test proxy',
          testingProxy: false,
        })
        throw error
      }
    },

    clearError: () => {
      set({ error: null })
    },

    getProviderById: (id: string) => {
      return get().providers.find(p => p.id === id)
    },

    getModelById: (id: string) => {
      const { providers } = get()
      for (const provider of providers) {
        const model = provider.models.find(m => m.id === id)
        if (model) return model
      }
      return undefined
    },
  })),
)
