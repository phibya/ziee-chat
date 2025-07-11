import { create } from 'zustand'
import { ConversationSummary } from '../types/api/chat'
import { ApiClient } from '../api/client'

interface ConversationsState {
  conversations: ConversationSummary[]
  isLoading: boolean

  // Actions
  loadConversations: () => Promise<void>
  addConversation: (conversation: ConversationSummary) => void
  updateConversation: (
    id: string,
    updates: Partial<ConversationSummary>,
  ) => void
  removeConversation: (id: string) => void
  setLoading: (loading: boolean) => void
}

export const useConversationsStore = create<ConversationsState>(set => ({
  conversations: [],
  isLoading: false,

  loadConversations: async () => {
    set({ isLoading: true })
    try {
      const response = await ApiClient.Chat.listConversations({
        page: 1,
        per_page: 20, // Show recent 20 conversations
      })
      set({ conversations: response.conversations })
    } catch (error) {
      console.error('Failed to load conversations:', error)
    } finally {
      set({ isLoading: false })
    }
  },

  addConversation: conversation => {
    set(state => ({
      conversations: [conversation, ...state.conversations],
    }))
  },

  updateConversation: (id, updates) => {
    set(state => ({
      conversations: state.conversations.map(conv =>
        conv.id === id ? { ...conv, ...updates } : conv,
      ),
    }))
  },

  removeConversation: id => {
    set(state => ({
      conversations: state.conversations.filter(conv => conv.id !== id),
    }))
  },

  setLoading: loading => {
    set({ isLoading: loading })
  },
}))
