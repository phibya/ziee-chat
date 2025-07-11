import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ConversationSummary } from '../types/api/chat'
import { ApiClient } from '../api/client'

interface ConversationsState {
  conversations: ConversationSummary[]
  isLoading: boolean
  error: string | null

  // Actions
  loadConversations: () => Promise<void>
  addConversation: (conversation: ConversationSummary) => void
  updateConversation: (
    id: string,
    updates: Partial<ConversationSummary>,
  ) => Promise<void>
  removeConversation: (id: string) => Promise<void>
  setLoading: (loading: boolean) => void
  clearError: () => void
}

export const useConversationsStore = create<ConversationsState>()(
  subscribeWithSelector(set => ({
    conversations: [],
    isLoading: false,
    error: null,

    loadConversations: async () => {
      set({ isLoading: true, error: null })
      try {
        const response = await ApiClient.Chat.listConversations({
          page: 1,
          per_page: 20, // Show recent 20 conversations
        })
        set({ conversations: response.conversations })
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to load conversations',
        })
      } finally {
        set({ isLoading: false })
      }
    },

    addConversation: conversation => {
      set(state => ({
        conversations: [conversation, ...state.conversations],
      }))
    },

    updateConversation: async (id, updates) => {
      try {
        set({ error: null })

        await ApiClient.Chat.updateConversation({
          conversation_id: id,
          ...updates,
        })

        set(state => ({
          conversations: state.conversations.map(conv =>
            conv.id === id ? { ...conv, ...updates } : conv,
          ),
        }))
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to update conversation',
        })
        throw error
      }
    },

    removeConversation: async id => {
      try {
        set({ error: null })

        await ApiClient.Chat.deleteConversation({ conversation_id: id })

        set(state => ({
          conversations: state.conversations.filter(conv => conv.id !== id),
        }))
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to delete conversation',
        })
        throw error
      }
    },

    setLoading: loading => {
      set({ isLoading: loading })
    },

    clearError: () => {
      set({ error: null })
    },
  })),
)
