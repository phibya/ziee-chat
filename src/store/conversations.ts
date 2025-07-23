import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ConversationSummary } from '../types/api/chat'
import { ApiClient } from '../api/client'

interface ConversationsState {
  conversations: ConversationSummary[]
  isLoading: boolean
  error: string | null
}

export const useConversationsStore = create<ConversationsState>()(
  subscribeWithSelector(
    (): ConversationsState => ({
      conversations: [],
      isLoading: false,
      error: null,
    }),
  ),
)

// Conversations actions
export const loadAllRecentConversations = async (): Promise<void> => {
  useConversationsStore.setState({ isLoading: true, error: null })
  try {
    const response = await ApiClient.Chat.listConversations({
      page: 1,
      per_page: 20, // Show recent 20 conversations
    })
    useConversationsStore.setState({ conversations: response.conversations })
  } catch (error) {
    useConversationsStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to load conversations',
    })
  } finally {
    useConversationsStore.setState({ isLoading: false })
  }
}

export const addNewConversationToList = (
  conversation: ConversationSummary,
): void => {
  useConversationsStore.setState(state => ({
    conversations: [conversation, ...state.conversations],
  }))
}

export const updateExistingConversation = async (
  id: string,
  updates: Partial<ConversationSummary>,
): Promise<void> => {
  try {
    useConversationsStore.setState({ error: null })

    await ApiClient.Chat.updateConversation({
      conversation_id: id,
      ...updates,
    })

    useConversationsStore.setState(state => ({
      conversations: state.conversations.map(conv =>
        conv.id === id ? { ...conv, ...updates } : conv,
      ),
    }))
  } catch (error) {
    useConversationsStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to update conversation',
    })
    throw error
  }
}

export const removeConversationFromList = async (id: string): Promise<void> => {
  try {
    useConversationsStore.setState({ error: null })

    await ApiClient.Chat.deleteConversation({ conversation_id: id })

    useConversationsStore.setState(state => ({
      conversations: state.conversations.filter(conv => conv.id !== id),
    }))
  } catch (error) {
    useConversationsStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to delete conversation',
    })
    throw error
  }
}

export const setConversationsListLoading = (loading: boolean): void => {
  useConversationsStore.setState({ isLoading: loading })
}

export const clearConversationsStoreError = (): void => {
  useConversationsStore.setState({ error: null })
}
