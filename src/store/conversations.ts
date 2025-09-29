import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ConversationSummary } from '../types'
import { ApiClient } from '../api/client'
import { ChatState, ChatStoreMap } from './chat.ts'

interface ConversationsState {
  conversations: ConversationSummary[]
  isInitialized: boolean
  isLoading: boolean
  updating: boolean
  deleting: boolean
  error: string | null

  __init__: {
    conversations: () => Promise<void>
  }
}

export const useConversationsStore = create<ConversationsState>()(
  subscribeWithSelector(
    (): ConversationsState => ({
      conversations: [],
      isInitialized: false,
      isLoading: false,
      updating: false,
      deleting: false,
      error: null,
      __init__: {
        conversations: () => loadAllRecentConversations(),
      },
    }),
  ),
)

// Conversations actions
export const loadAllRecentConversations = async (): Promise<void> => {
  const state = useConversationsStore.getState()
  if (state.isInitialized || state.isLoading) {
    return
  }
  useConversationsStore.setState({ isLoading: true, error: null })
  try {
    const response = await ApiClient.Conversation.listConversations({
      page: 1,
      per_page: 20, // Show recent 20 conversations
    })
    useConversationsStore.setState({
      conversations: response.conversations,
      isInitialized: true,
      isLoading: false,
    })
  } catch (error) {
    useConversationsStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to load conversations',
      isLoading: false,
    })
    throw error
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
  const state = useConversationsStore.getState()
  if (state.updating) {
    return
  }

  try {
    useConversationsStore.setState({ updating: true, error: null })

    await ApiClient.Conversation.updateConversation({
      conversation_id: id,
      ...updates,
    })

    useConversationsStore.setState(state => ({
      conversations: state.conversations.map(conv =>
        conv.id === id ? { ...conv, ...updates } : conv,
      ),
      updating: false,
    }))

    const chatStore = ChatStoreMap.get(id)
    if (chatStore) {
      chatStore.__setState(
        state =>
          ({
            conversation: {
              ...state.conversation,
              ...updates,
            },
          }) as Partial<ChatState>,
      )
    }
  } catch (error) {
    useConversationsStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to update conversation',
      updating: false,
    })
    throw error
  }
}

export const removeConversationFromList = async (id: string): Promise<void> => {
  const state = useConversationsStore.getState()
  if (state.deleting) {
    return
  }

  try {
    useConversationsStore.setState({ deleting: true, error: null })

    await ApiClient.Conversation.deleteConversation({ conversation_id: id })

    useConversationsStore.setState(state => ({
      conversations: state.conversations.filter(conv => conv.id !== id),
      deleting: false,
    }))
  } catch (error) {
    useConversationsStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to delete conversation',
      deleting: false,
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
