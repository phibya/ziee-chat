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
}

export const useChatHistoryStore = create<ChatHistoryState>()(
  subscribeWithSelector(
    (): ChatHistoryState => ({
      // Initial state
      conversations: [],
      searchResults: [],
      searchQuery: '',
      isSearching: false,
      loading: false,
      deleting: false,
      clearing: false,
      error: null,
    }),
  ),
)

// Chat history actions
export const loadChatHistoryConversationsList = async (): Promise<void> => {
  try {
    useChatHistoryStore.setState({ loading: true, error: null })

    const response = await ApiClient.Chat.listConversations({
      page: 1,
      per_page: 50,
    })

    useChatHistoryStore.setState({
      conversations: response.conversations,
      loading: false,
    })
  } catch (error) {
    useChatHistoryStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to load conversations',
      loading: false,
    })
    throw error
  }
}

export const searchChatHistoryConversations = async (query: string): Promise<void> => {
  try {
    useChatHistoryStore.setState({
      isSearching: true,
      searchQuery: query,
      error: null,
    })

    if (!query.trim()) {
      useChatHistoryStore.setState({
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

    useChatHistoryStore.setState({
      searchResults: response.conversations,
      isSearching: false,
    })
  } catch (error) {
    useChatHistoryStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to search conversations',
      isSearching: false,
    })
    throw error
  }
}

export const deleteChatHistoryConversationById = async (
  id: string,
): Promise<void> => {
  try {
    useChatHistoryStore.setState({ deleting: true, error: null })

    await ApiClient.Chat.deleteConversation({ conversation_id: id })

    useChatHistoryStore.setState(state => ({
      conversations: state.conversations.filter(c => c.id !== id),
      searchResults: state.searchResults.filter(c => c.id !== id),
      deleting: false,
    }))
  } catch (error) {
    useChatHistoryStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to delete conversation',
      deleting: false,
    })
    throw error
  }
}

export const clearAllUserChatHistoryConversations = async (): Promise<void> => {
  try {
    useChatHistoryStore.setState({ clearing: true, error: null })

    await ApiClient.Chat.clearAllConversations()

    useChatHistoryStore.setState({
      conversations: [],
      searchResults: [],
      clearing: false,
    })
  } catch (error) {
    useChatHistoryStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to clear conversations',
      clearing: false,
    })
    throw error
  }
}

export const updateChatHistoryConversationTitleById = async (
  id: string,
  title: string,
): Promise<void> => {
  try {
    useChatHistoryStore.setState({ error: null })

    await ApiClient.Chat.updateConversation({ conversation_id: id, title })

    useChatHistoryStore.setState(state => ({
      conversations: state.conversations.map(c =>
        c.id === id ? { ...c, title } : c,
      ),
      searchResults: state.searchResults.map(c =>
        c.id === id ? { ...c, title } : c,
      ),
    }))
  } catch (error) {
    useChatHistoryStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to update conversation title',
    })
    throw error
  }
}

export const clearChatHistorySearchResults = (): void => {
  useChatHistoryStore.setState({
    searchQuery: '',
    searchResults: [],
    isSearching: false,
  })
}

export const clearChatHistoryStoreError = (): void => {
  useChatHistoryStore.setState({ error: null })
}
