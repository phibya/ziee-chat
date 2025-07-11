import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../api/client'
import { ConversationSummary } from '../types/api/chat'

interface ChatHistoryState {
  // Data
  conversations: ConversationSummary[]
  searchResults: ConversationSummary[]

  // Search state
  searchQuery: string
  isSearching: boolean

  // Loading states
  loading: boolean
  deleting: boolean
  clearing: boolean

  // Error state
  error: string | null

  // Actions
  loadConversations: () => Promise<void>
  searchConversations: (query: string) => Promise<void>
  deleteConversation: (id: string) => Promise<void>
  clearAllConversations: () => Promise<void>
  updateConversationTitle: (id: string, title: string) => Promise<void>
  clearSearch: () => void
  clearError: () => void
}

export const useChatHistoryStore = create<ChatHistoryState>()(
  subscribeWithSelector(set => ({
    // Initial state
    conversations: [],
    searchResults: [],
    searchQuery: '',
    isSearching: false,
    loading: false,
    deleting: false,
    clearing: false,
    error: null,

    loadConversations: async () => {
      try {
        set({ loading: true, error: null })

        const response = await ApiClient.Chat.listConversations({
          page: 1,
          per_page: 50,
        })

        set({
          conversations: response.conversations,
          loading: false,
        })
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to load conversations',
          loading: false,
        })
        throw error
      }
    },

    searchConversations: async (query: string) => {
      try {
        set({ isSearching: true, searchQuery: query, error: null })

        if (!query.trim()) {
          set({
            searchResults: [],
            isSearching: false,
            searchQuery: '',
          })
          return
        }

        const response = await ApiClient.Chat.searchConversations({
          q: query,
          page: 1,
          per_page: 50,
        })

        set({
          searchResults: response.conversations,
          isSearching: false,
        })
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to search conversations',
          isSearching: false,
        })
        throw error
      }
    },

    deleteConversation: async (id: string) => {
      try {
        set({ deleting: true, error: null })

        await ApiClient.Chat.deleteConversation({ conversation_id: id })

        set(state => ({
          conversations: state.conversations.filter(c => c.id !== id),
          searchResults: state.searchResults.filter(c => c.id !== id),
          deleting: false,
        }))
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to delete conversation',
          deleting: false,
        })
        throw error
      }
    },

    clearAllConversations: async () => {
      try {
        set({ clearing: true, error: null })

        await ApiClient.Chat.clearAllConversations()

        set({
          conversations: [],
          searchResults: [],
          clearing: false,
        })
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to clear conversations',
          clearing: false,
        })
        throw error
      }
    },

    updateConversationTitle: async (id: string, title: string) => {
      try {
        set({ error: null })

        await ApiClient.Chat.updateConversation({ conversation_id: id, title })

        set(state => ({
          conversations: state.conversations.map(c =>
            c.id === id ? { ...c, title } : c,
          ),
          searchResults: state.searchResults.map(c =>
            c.id === id ? { ...c, title } : c,
          ),
        }))
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to update conversation title',
        })
        throw error
      }
    },

    clearSearch: () => {
      set({
        searchQuery: '',
        searchResults: [],
        isSearching: false,
      })
    },

    clearError: () => {
      set({ error: null })
    },
  })),
)
