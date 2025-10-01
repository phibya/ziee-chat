import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../api/client'
import {
  Conversation,
  Message,
  MessageContentDataText,
  ChatMessageRequest,
} from '../types'
import { useConversationsStore } from './conversations.ts'
import { getFile } from './files.ts'
import { createStoreProxy } from '../utils/createStoreProxy.ts'
import { StoreApi, UseBoundStore } from 'zustand/index'
import { useEffect, useMemo, useRef } from 'react'
import { useParams } from 'react-router-dom'
import { debounce } from '../utils/debounce'
import { createSSEHandlers } from './chat/sseHandlers'

// Helper function to get text content from structured message contents
export const getMessageText = (message: Message): string => {
  return message.contents
    .filter(c => c.content_type === 'text')
    .map(c => (c.content as MessageContentDataText).text)
    .join('\n')
}

// Helper function to create structured text content
const createTextContent = (text: string) => [
  {
    id: crypto.randomUUID(),
    message_id: '', // Will be set when message is created
    content_type: 'text' as const,
    content: { text },
    sequence_order: 0,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  },
]

const BranchMessagesCacheMap = new Map<string, Message[]>()

// Map to track cleanup debounce functions for inactive branches
const BranchCleanupDebounceMap = new Map<string, ReturnType<typeof debounce>>()

// Helper function to cache current branch and set up cleanup for the previous one
const cacheBranchAndSetupCleanup = (
  conversationId: string,
  currentBranchId: string,
  targetBranchId: string,
  currentMessages: Message[],
) => {
  if (currentBranchId && currentBranchId !== targetBranchId) {
    const currentCacheKey = `${conversationId}:${currentBranchId}`
    BranchMessagesCacheMap.set(currentCacheKey, currentMessages)

    // Set up debounced cleanup for the previous branch
    const cleanupFn = debounce(() => {
      BranchMessagesCacheMap.delete(currentCacheKey)
      BranchCleanupDebounceMap.delete(currentCacheKey)
    }, 60 * 1000) // 1 minute

    BranchCleanupDebounceMap.set(currentCacheKey, cleanupFn)
    cleanupFn()
  }
}

// Helper function to cancel cleanup for an active branch
const cancelBranchCleanup = (cacheKey: string) => {
  const existingCleanup = BranchCleanupDebounceMap.get(cacheKey)
  if (existingCleanup) {
    existingCleanup.cancel()
    BranchCleanupDebounceMap.delete(cacheKey)
  }
}

export interface ChatState {
  // Current conversation state
  conversation: Conversation | null
  messages: Message[]
  activeBranchId: string | null

  // Loading states
  loading: boolean
  sending: boolean
  loadingBranches: boolean

  // Error state
  error: string | null

  // Stream state
  isStreaming: boolean

  // Store management
  destroy: () => void

  // Actions
  loadConversation: () => Promise<void>
  loadMessages: (branchId?: string) => Promise<void>
  sendMessage: (
    params: Omit<ChatMessageRequest, 'conversation_id'>,
  ) => Promise<void>
  editMessage: (
    messageId: string,
    params: Omit<ChatMessageRequest, 'conversation_id'>,
  ) => Promise<void>
  switchBranch: (branchId: string) => Promise<void>
  stopStreaming: () => void
  clearError: () => void
  reset: () => void
}

// Store map to keep the proxies
export const ChatStoreMap = new Map<
  string,
  ReturnType<typeof createStoreProxy<UseBoundStore<StoreApi<ChatState>>>>
>()

// Map to track cleanup debounce functions for each conversation
const CleanupDebounceMap = new Map<string, ReturnType<typeof debounce>>()

export const createChatStore = (conversation: string | Conversation) => {
  let conversationId: string
  if (typeof conversation === 'string') {
    conversationId = conversation
  } else {
    conversationId = conversation.id
  }

  if (ChatStoreMap.has(conversationId)) {
    return ChatStoreMap.get(conversationId)!
  }

  const store = create<ChatState>()(
    subscribeWithSelector(
      (set, get): ChatState => ({
        // Initial state
        conversation: typeof conversation === 'string' ? null : conversation,
        messages: [],
        activeBranchId:
          typeof conversation === 'string'
            ? null
            : conversation.active_branch_id || null,
        loading: false,
        sending: false,
        loadingBranches: false,
        error: null,
        isStreaming: false,

        destroy: () => {
          // Clean up cached messages and debounce timers for this conversation
          const keysToDelete: string[] = []
          BranchMessagesCacheMap.forEach((_, key) => {
            if (key.startsWith(`${conversationId}:`)) {
              keysToDelete.push(key)
            }
          })
          keysToDelete.forEach(key => {
            BranchMessagesCacheMap.delete(key)
            const cleanup = BranchCleanupDebounceMap.get(key)
            if (cleanup) {
              cleanup.cancel()
              BranchCleanupDebounceMap.delete(key)
            }
          })

          // Remove the store from the map and let the browser GC it
          ChatStoreMap.delete(conversationId)
        },

        // Actions
        loadConversation: async () => {
          const state = get()
          if (state.loading) {
            return
          }
          try {
            set({ loading: true, error: null })

            // Get conversation info
            const conversation = await ApiClient.Conversation.getConversation({
              conversation_id: conversationId,
            })

            useConversationsStore.setState(state => ({
              conversations: state.conversations.map(conv => {
                if (conv.id === conversationId) {
                  return {
                    ...conv,
                    title: conversation.title || conv.title,
                  }
                }
                return conv
              }),
            }))

            set({
              conversation: conversation,
              activeBranchId: conversation.active_branch_id,
              loading: false,
            })
          } catch (error) {
            set({
              error:
                error instanceof Error
                  ? error.message
                  : 'Failed to load conversation',
              loading: false,
            })
            throw error
          }
        },

        loadMessages: async (branchId?: string) => {
          try {
            const { conversation, activeBranchId: currentBranchId } = get()
            if (!conversation) {
              throw new Error('No conversation loaded')
            }

            set({ loading: !get().messages.length, error: null })

            const targetBranchId = branchId || conversation.active_branch_id
            if (!targetBranchId) {
              throw new Error('No branch ID available')
            }

            const cacheKey = `${conversationId}:${targetBranchId}`

            // Cache current branch messages before switching
            if (currentBranchId) {
              cacheBranchAndSetupCleanup(
                conversationId,
                currentBranchId,
                targetBranchId,
                get().messages,
              )
            }

            // Cancel any existing cleanup for the target branch since it's being accessed
            cancelBranchCleanup(cacheKey)

            // Check if messages are cached
            const cachedMessages = BranchMessagesCacheMap.get(cacheKey)
            if (cachedMessages) {
              set({
                messages: cachedMessages,
                activeBranchId: targetBranchId,
                loading: false,
              })
              return
            }

            // Load messages from API if not cached
            const messages =
              await ApiClient.Chat.getConversationMessagesByBranch({
                conversation_id: conversationId,
                branch_id: targetBranchId,
              })

            // Cache the loaded messages
            BranchMessagesCacheMap.set(cacheKey, messages)

            set({
              messages: messages,
              activeBranchId: targetBranchId,
              loading: false,
            })
          } catch (error) {
            set({
              error:
                error instanceof Error
                  ? error.message
                  : 'Failed to load messages',
              loading: false,
            })
            throw error
          }
        },

        sendMessage: async params => {
          const { activeBranchId, sending } = get()
          if (!conversationId || !activeBranchId) return
          if (sending) {
            return
          }

          try {
            set({
              sending: true,
              error: null,
              isStreaming: true,
            })

            // Track actual message IDs from server (using mutable refs)
            const actualUserMessageId = { current: null as string | null }
            const actualAssistantMessageId = { current: null as string | null }
            const userMessage = { current: null as Message | null }
            const assistantMessage = { current: null as Message | null }

            // Only create new messages if not resuming from existing message
            if (!params.message_id) {
              const files = await Promise.all(
                (params.file_ids || []).map(getFile),
              )

              // Add user message immediately
              const userMessageId = crypto.randomUUID()
              userMessage.current = {
                id: userMessageId,
                conversation_id: conversationId,
                contents: createTextContent(params.content).map(c => ({
                  ...c,
                  message_id: userMessageId,
                })),
                role: 'user',
                created_at: new Date().toISOString(),
                updated_at: new Date().toISOString(),
                edit_count: 0,
                originated_from_id: crypto.randomUUID(),
                files: files,
              }

              set(state => {
                const newMessages = [...state.messages, userMessage.current!]
                // Update cache when adding new message
                if (activeBranchId) {
                  const cacheKey = `${conversationId}:${activeBranchId}`
                  BranchMessagesCacheMap.set(cacheKey, newMessages)
                }
                return { messages: newMessages }
              })

              // Create assistant message placeholder
              const assistantMessageId = crypto.randomUUID()
              assistantMessage.current = {
                id: assistantMessageId,
                conversation_id: conversationId,
                contents: createTextContent('').map(c => ({
                  ...c,
                  message_id: assistantMessageId,
                })),
                role: 'assistant',
                created_at: new Date().toISOString(),
                updated_at: new Date().toISOString(),
                edit_count: 0,
                originated_from_id: crypto.randomUUID(),
                files: [],
              }

              set(state => {
                const newMessages = [
                  ...state.messages,
                  assistantMessage.current!,
                ]
                // Update cache when adding assistant message placeholder
                if (activeBranchId) {
                  const cacheKey = `${conversationId}:${activeBranchId}`
                  BranchMessagesCacheMap.set(cacheKey, newMessages)
                }
                return { messages: newMessages }
              })
            } else {
              // Resuming from existing message - find it in the current messages
              const existingMessage = get().messages.find(
                (msg: Message) => msg.id === params.message_id
              )
              if (existingMessage) {
                assistantMessage.current = existingMessage
                actualAssistantMessageId.current = existingMessage.id
              }
            }

            // Create unified SSE handlers
            const handlers = createSSEHandlers({
              set,
              get,
              conversationId,
              activeBranchId,
              BranchMessagesCacheMap,
              getTargetMessageId: () =>
                actualAssistantMessageId.current ||
                assistantMessage.current?.id ||
                null,
              actualUserMessageId,
              actualAssistantMessageId,
              userMessage,
              assistantMessage,
            })

            // Send message with streaming using unified handlers
            await ApiClient.Chat.sendMessageStream(
              {
                conversation_id: conversationId,
                content: params.content,
                model_id: params.model_id,
                assistant_id: params.assistant_id,
                file_ids: params.file_ids,
                enabled_tools: params.enabled_tools,
                message_id: params.message_id,
              },
              { SSE: handlers },
            )

            set({
              sending: false,
              isStreaming: false,
            })
          } catch (error) {
            set({
              error:
                error instanceof Error
                  ? error.message
                  : 'Failed to send message',
              sending: false,
              isStreaming: false,
            })
            throw error
          }
        },

        editMessage: async (messageId: string, params) => {
          const { conversation, activeBranchId, sending } = get()
          if (!conversation) return
          if (sending) {
            return
          }

          try {
            set({
              sending: true,
              error: null,
              isStreaming: true,
            })

            const currentMessage = get().messages.find(
              (msg: Message) => msg.id === messageId,
            )

            if (!currentMessage) {
              throw new Error('Message not found')
            }

            const files = await Promise.all(
              (params.file_ids || []).map(getFile),
            )

            // Update the user message immediately with the new content
            set(state => {
              let currentMessages = state.messages.filter(
                (m: Message) =>
                  new Date(m.created_at) <= new Date(currentMessage.created_at),
              )

              return {
                messages: currentMessages.map((msg: Message) =>
                  msg.id === messageId
                    ? {
                        ...msg,
                        contents: createTextContent(params.content).map(c => ({
                          ...c,
                          message_id: messageId,
                        })),
                        files,
                      }
                    : msg,
                ),
              }
            })

            // Create assistant message placeholder for streaming
            const tempAssistantMessage: Message = {
              id: 'streaming-temp',
              conversation_id: conversation.id,
              contents: createTextContent('').map(c => ({
                ...c,
                message_id: 'streaming-temp',
              })),
              role: 'assistant',
              created_at: new Date().toISOString(),
              updated_at: new Date().toISOString(),
              edit_count: 0,
              originated_from_id: messageId,
              files: [],
            }

            set(state => ({
              messages: [...state.messages, tempAssistantMessage],
            }))

            // Track actual message ID from server (using mutable refs)
            const actualAssistantMessageId = { current: null as string | null }
            const assistantMessage = { current: tempAssistantMessage }

            // Create unified SSE handlers
            const handlers = createSSEHandlers({
              set,
              get,
              conversationId: conversation.id,
              activeBranchId,
              BranchMessagesCacheMap,
              getTargetMessageId: () =>
                actualAssistantMessageId.current || assistantMessage.current?.id || null,
              actualUserMessageId: { current: null }, // Not used by edit
              actualAssistantMessageId,
              userMessage: { current: null }, // Not used by edit
              assistantMessage,
              editMessageId: messageId, // For editedMessage handler
            })

            // Use streaming edit endpoint with unified handlers
            await ApiClient.Chat.editMessageStream(
              {
                message_id: messageId,
                conversation_id: conversation.id,
                model_id: params.model_id,
                assistant_id: params.assistant_id,
                content: params.content,
                file_ids: params.file_ids,
                enabled_tools: params.enabled_tools,
              },
              { SSE: handlers },
            )

            set({
              sending: false,
              isStreaming: false,
            })
          } catch (error) {
            set({
              error:
                error instanceof Error
                  ? error.message
                  : 'Failed to edit message',
              sending: false,
              isStreaming: false,
              // Remove the streaming placeholder on error
              messages: get().messages.filter(
                (msg: Message) => msg.id !== 'streaming-temp',
              ),
            })
            throw error
          }
        },

        switchBranch: async (branchId: string) => {
          try {
            const { activeBranchId: currentBranchId } = get()
            set({ error: null })

            // Cache current branch messages before switching
            if (currentBranchId) {
              cacheBranchAndSetupCleanup(
                conversationId,
                currentBranchId,
                branchId,
                get().messages,
              )
            }

            await ApiClient.Conversation.switchConversationBranch({
              conversation_id: conversationId,
              branch_id: branchId,
            })

            // After switching, reload the conversation and get messages for the new branch
            // The loadMessages function will handle caching automatically
            await get().loadMessages(branchId)
            set({
              activeBranchId: branchId,
            })
          } catch (error) {
            set({
              error:
                error instanceof Error
                  ? error.message
                  : 'Failed to switch branch',
              loading: false,
            })
            throw error
          }
        },

        stopStreaming: () => {
          set({ isStreaming: false, sending: false })
        },

        clearError: () => {
          set({ error: null })
        },

        reset: () => {
          set({
            conversation: null,
            messages: [],
            activeBranchId: null,
            loading: false,
            sending: false,
            loadingBranches: false,
            error: null,
            isStreaming: false,
          })
        },
      }),
    ),
  )

  const storeProxy = createStoreProxy(store)
  ChatStoreMap.set(conversationId, storeProxy)

  // Immediately load conversation and messages when store is created (except for default store)
  if (conversationId !== 'default') {
    storeProxy.__state
      .loadConversation()
      .then(() => storeProxy.__state.loadMessages())
      .catch(error => {
        console.error(
          `Failed to auto-load conversation ${conversationId}:`,
          error,
        )
      })
  }

  return storeProxy
}

// Hook for components to use conversation-specific stores
export const useChatStore = (conversationId?: string) => {
  const { conversationId: paramConversationId } = useParams<{
    conversationId?: string
  }>()
  const currentId = conversationId || paramConversationId
  const prevIdRef = useRef<string | undefined>(currentId)

  useEffect(() => {
    const prevId = prevIdRef.current

    // If conversationId changed, set up debounced cleanup for the previous one
    if (prevId && prevId !== currentId) {
      // Create debounced cleanup function for the previous conversation
      const cleanupFn = debounce(
        () => {
          const store = ChatStoreMap.get(prevId)
          if (store) {
            store.__state.destroy()
          }
          CleanupDebounceMap.delete(prevId)
        },
        5 * 60 * 1000,
      ) // 5 minutes

      CleanupDebounceMap.set(prevId, cleanupFn)
      cleanupFn()
    }

    // Update the ref for the next render
    prevIdRef.current = currentId
  }, [currentId])

  return useMemo(() => {
    const id = currentId
    if (!id) {
      // Return a default store for cases where there's no conversation ID
      return createChatStore('default')
    }

    // Cancel any existing debounced cleanup for this conversation since it's being accessed again
    const existingCleanup = CleanupDebounceMap.get(id)
    if (existingCleanup) {
      existingCleanup.cancel()
      CleanupDebounceMap.delete(id)
    }

    return createChatStore(id)
  }, [conversationId, paramConversationId])
}

// this function is independent of the chat store
export const createNewConversation = async (
  assistantId: string,
  modelId: string,
  projectId?: string,
): Promise<Conversation> => {
  return await ApiClient.Conversation.createConversation({
    title: 'New Conversation', // This will be auto-generated by the backend
    assistant_id: assistantId,
    model_id: modelId,
    project_id: projectId,
  })
}
