import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../api/client'
import {
  Conversation,
  Message,
  MessageBranch,
  MessageContentDataText,
  MessageContentDataToolCall,
  MessageContentDataToolCallPendingApproval,
  MessageContentDataToolResult,
  MessageContentItem,
  ConnectedData,
  CompleteData,
  MessageContentChunkData,
  NewUserMessageData,
  ToolCallData,
  ToolCallPendingApprovalData,
  ToolResultData,
  TitleUpdatedData,
  ChatMessageRequest,
} from '../types'
import {
  useConversationsStore,
  updateConversationTitle,
} from './conversations.ts'
import { getFile } from './files.ts'
import { createStoreProxy } from '../utils/createStoreProxy.ts'
import { StoreApi, UseBoundStore } from 'zustand/index'
import { useEffect, useMemo, useRef } from 'react'
import { useParams } from 'react-router-dom'
import { debounce } from '../utils/debounce'
import { removeMessageBranchStoreByOriginatedId } from './messageBranches.ts'

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

            // Track actual message IDs from server
            let actualUserMessageId: string | null = null
            let actualAssistantMessageId: string | null = null
            let userMessage: Message | null = null
            let assistantMessage: Message | null = null

            // Only create new messages if not resuming from existing message
            if (!params.message_id) {
              const files = await Promise.all(
                (params.file_ids || []).map(getFile),
              )

              // Add user message immediately
              const userMessageId = crypto.randomUUID()
              userMessage = {
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
                const newMessages = [...state.messages, userMessage!]
                // Update cache when adding new message
                if (activeBranchId) {
                  const cacheKey = `${conversationId}:${activeBranchId}`
                  BranchMessagesCacheMap.set(cacheKey, newMessages)
                }
                return { messages: newMessages }
              })

              // Create assistant message placeholder
              const assistantMessageId = crypto.randomUUID()
              assistantMessage = {
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
                const newMessages = [...state.messages, assistantMessage!]
                // Update cache when adding assistant message placeholder
                if (activeBranchId) {
                  const cacheKey = `${conversationId}:${activeBranchId}`
                  BranchMessagesCacheMap.set(cacheKey, newMessages)
                }
                return { messages: newMessages }
              })
            }

            // Send message with streaming
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
              {
                SSE: {
                  connected: (_data: ConnectedData) => {
                    console.log('Chat stream connected')
                  },
                  newUserMessage: (data: NewUserMessageData) => {
                    // Server has created the user message, track the ID
                    actualUserMessageId = data.message_id

                    // If we created a temporary user message, update it with the server ID
                    if (userMessage) {
                      set(state => {
                        const updatedMessages = state.messages.map(msg => {
                          if (msg.id === userMessage!.id) {
                            return {
                              ...msg,
                              id: actualUserMessageId!,
                              contents: msg.contents.map(c => ({
                                ...c,
                                message_id: actualUserMessageId!,
                              })),
                            } as Message
                          }
                          return msg
                        })

                        // Update cache with new message ID
                        if (activeBranchId) {
                          const cacheKey = `${conversationId}:${activeBranchId}`
                          BranchMessagesCacheMap.set(cacheKey, updatedMessages)
                        }

                        return { messages: updatedMessages }
                      })
                    }
                  },
                  newAssistantMessage: data => {
                    // Server has created the assistant message, track the ID
                    actualAssistantMessageId = data.message_id

                    // If we created a temporary assistant message, update it with the server ID
                    if (assistantMessage) {
                      set(state => {
                        const updatedMessages = state.messages.map(msg => {
                          if (msg.id === assistantMessage!.id) {
                            return {
                              ...msg,
                              id: actualAssistantMessageId!,
                              contents: msg.contents.map(c => ({
                                ...c,
                                message_id: actualAssistantMessageId!,
                              })),
                            } as Message
                          }
                          return msg
                        })

                        // Update cache with new message ID
                        if (activeBranchId) {
                          const cacheKey = `${conversationId}:${activeBranchId}`
                          BranchMessagesCacheMap.set(cacheKey, updatedMessages)
                        }

                        return { messages: updatedMessages }
                      })
                    }
                  },
                  messageContentChunk: (data: MessageContentChunkData) => {
                    // Append delta directly to the assistant message
                    if (data.delta) {
                      set(state => {
                        const targetMessageId =
                          actualAssistantMessageId || assistantMessage?.id
                        if (!targetMessageId) return {}

                        const updatedMessages = state.messages.map(msg => {
                          if (msg.id === targetMessageId) {
                            // Find the text content and append delta
                            const textContent = msg.contents.find(
                              c => c.content_type === 'text',
                            )
                            if (textContent) {
                              const updatedContents = msg.contents.map(c => {
                                if (c.content_type === 'text') {
                                  return {
                                    ...c,
                                    content: {
                                      ...c.content,
                                      text:
                                        (c.content as MessageContentDataText)
                                          .text + data.delta,
                                    },
                                  }
                                }
                                return c
                              })
                              return { ...msg, contents: updatedContents }
                            }
                          }
                          return msg
                        })
                        return { messages: updatedMessages }
                      })
                    }
                  },
                  toolCall: (data: ToolCallData) => {
                    // Add tool call content to the assistant message
                    set(state => {
                      const targetMessageId =
                        actualAssistantMessageId || assistantMessage?.id
                      if (!targetMessageId) return {}

                      const updatedMessages = state.messages.map(msg => {
                        if (msg.id === targetMessageId) {
                          const newContent: MessageContentItem = {
                            id: data.message_content_id,
                            message_id: data.message_id,
                            content_type: 'tool_call',
                            content: {
                              call_id: data.call_id,
                              tool_name: data.tool_name,
                              server_id: data.server_id,
                              arguments: data.arguments,
                            } as MessageContentDataToolCall,
                            sequence_order: msg.contents.length,
                            created_at: new Date().toISOString(),
                            updated_at: new Date().toISOString(),
                          }
                          return {
                            ...msg,
                            contents: [...msg.contents, newContent],
                          }
                        }
                        return msg
                      })
                      return { messages: updatedMessages }
                    })
                  },
                  toolCallPendingApproval: (
                    data: ToolCallPendingApprovalData,
                  ) => {
                    // Add tool call pending approval content to the assistant message
                    set(state => {
                      const targetMessageId =
                        actualAssistantMessageId || assistantMessage?.id
                      if (!targetMessageId) return {}

                      const updatedMessages = state.messages.map(msg => {
                        if (msg.id === targetMessageId) {
                          const newContent: MessageContentItem = {
                            id: data.message_content_id,
                            message_id: data.message_id,
                            content_type: 'tool_call_pending_approval',
                            content: {
                              tool_name: data.tool_name,
                              server_id: data.server_id,
                              arguments: data.arguments,
                            } as MessageContentDataToolCallPendingApproval,
                            sequence_order: msg.contents.length,
                            created_at: new Date().toISOString(),
                            updated_at: new Date().toISOString(),
                          }
                          return {
                            ...msg,
                            contents: [...msg.contents, newContent],
                          }
                        }
                        return msg
                      })
                      return {
                        messages: updatedMessages,
                        isStreaming: false,
                        sending: false,
                      }
                    })
                  },
                  toolResult: (data: ToolResultData) => {
                    // Add tool result content to the assistant message
                    set(state => {
                      const targetMessageId =
                        actualAssistantMessageId || assistantMessage?.id
                      if (!targetMessageId) return {}

                      const updatedMessages = state.messages.map(msg => {
                        if (msg.id === targetMessageId) {
                          const newContent: MessageContentItem = {
                            id: data.message_content_id,
                            message_id: data.message_id,
                            content_type: 'tool_result',
                            content: {
                              call_id: data.call_id,
                              result: data.result,
                              success: data.success,
                              error_message: data.error_message,
                            } as MessageContentDataToolResult,
                            sequence_order: msg.contents.length,
                            created_at: new Date().toISOString(),
                            updated_at: new Date().toISOString(),
                          }
                          return {
                            ...msg,
                            contents: [...msg.contents, newContent],
                          }
                        }
                        return msg
                      })
                      return { messages: updatedMessages }
                    })
                  },
                  titleUpdated: (data: TitleUpdatedData) => {
                    // Update conversation title in the chat store
                    set(state => {
                      if (state.conversation) {
                        return {
                          conversation: {
                            ...state.conversation,
                            title: data.title,
                          },
                        }
                      }
                      return {}
                    })

                    // Also update in conversations list
                    updateConversationTitle(conversationId, data.title)
                  },
                  complete: (_data: CompleteData) => {
                    // Handle completion events - update with actual message ID from server
                    set(state => {
                      const targetMessageId = assistantMessage?.id
                      if (!targetMessageId || !actualAssistantMessageId) {
                        return {}
                      }

                      const updatedMessages: Message[] = state.messages.map(
                        msg => {
                          if (msg.id === targetMessageId) {
                            return {
                              ...msg,
                              id: actualAssistantMessageId,
                              contents: msg.contents.map(c => ({
                                ...c,
                                message_id: actualAssistantMessageId,
                              })),
                            } as Message
                          }
                          return msg
                        },
                      )
                      const newMessages = updatedMessages

                      // Update cache when streaming is complete
                      if (activeBranchId) {
                        const cacheKey = `${conversationId}:${activeBranchId}`
                        BranchMessagesCacheMap.set(cacheKey, newMessages)
                      }

                      return {
                        isStreaming: false,
                        sending: false,
                        messages: newMessages,
                      }
                    })
                  },
                  error: data => {
                    set({
                      error: data.error,
                      sending: false,
                      isStreaming: false,
                    })
                    console.error('Streaming error:', data)
                  },
                  default: (event, data) => {
                    console.log('Unknown chat stream SSE event:', event, data)
                  },
                },
              },
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
          const { conversation, sending } = get()
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
            const assistantMessage: Message = {
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
              messages: [...state.messages, assistantMessage],
            }))

            // Track actual message ID from server
            let actualAssistantMessageId: string | null = null

            // Use streaming edit endpoint
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
              {
                SSE: {
                  connected: (_data: ConnectedData) => {
                    console.log('Chat edit stream connected')
                  },
                  newAssistantMessage: data => {
                    // Server has created the assistant message, track the ID
                    actualAssistantMessageId = data.message_id
                  },
                  editedMessage: data => {
                    let editedMessage = data as Message
                    // find the message.id === messageId and replace it with editedMessage
                    set(state => ({
                      messages: state.messages.map((msg: Message) =>
                        msg.id === messageId ? editedMessage : msg,
                      ),
                    }))
                    removeMessageBranchStoreByOriginatedId(
                      editedMessage.originated_from_id,
                    )
                  },
                  createdBranch: data => {
                    // Handle branch creation events
                    const newBranch = data as MessageBranch
                    set({
                      activeBranchId: newBranch.id,
                    })
                  },
                  messageContentChunk: (data: MessageContentChunkData) => {
                    // Append delta directly to the streaming-temp assistant message
                    if (data.delta) {
                      set(state => {
                        const updatedMessages = state.messages.map(msg => {
                          if (msg.id === 'streaming-temp') {
                            const textContent = msg.contents.find(
                              c => c.content_type === 'text',
                            )
                            if (textContent) {
                              const updatedContents = msg.contents.map(c => {
                                if (c.content_type === 'text') {
                                  return {
                                    ...c,
                                    content: {
                                      ...c.content,
                                      text:
                                        (c.content as MessageContentDataText)
                                          .text + data.delta,
                                    },
                                  }
                                }
                                return c
                              })
                              return { ...msg, contents: updatedContents }
                            }
                          }
                          return msg
                        })
                        return { messages: updatedMessages }
                      })
                    }
                  },
                  toolCall: (data: ToolCallData) => {
                    // Add tool call content to the streaming-temp assistant message
                    set(state => {
                      const updatedMessages = state.messages.map(msg => {
                        if (msg.id === 'streaming-temp') {
                          const newContent: MessageContentItem = {
                            id: data.message_content_id,
                            message_id: data.message_id,
                            content_type: 'tool_call',
                            content: {
                              call_id: data.call_id,
                              tool_name: data.tool_name,
                              server_id: data.server_id,
                              arguments: data.arguments,
                            } as MessageContentDataToolCall,
                            sequence_order: msg.contents.length,
                            created_at: new Date().toISOString(),
                            updated_at: new Date().toISOString(),
                          }
                          return {
                            ...msg,
                            contents: [...msg.contents, newContent],
                          }
                        }
                        return msg
                      })
                      return { messages: updatedMessages }
                    })
                  },
                  toolCallPendingApproval: (
                    data: ToolCallPendingApprovalData,
                  ) => {
                    // Add tool call pending approval content to the streaming-temp assistant message
                    set(state => {
                      const updatedMessages = state.messages.map(msg => {
                        if (msg.id === 'streaming-temp') {
                          const newContent: MessageContentItem = {
                            id: data.message_content_id,
                            message_id: data.message_id,
                            content_type: 'tool_call_pending_approval',
                            content: {
                              tool_name: data.tool_name,
                              server_id: data.server_id,
                              arguments: data.arguments,
                            } as MessageContentDataToolCallPendingApproval,
                            sequence_order: msg.contents.length,
                            created_at: new Date().toISOString(),
                            updated_at: new Date().toISOString(),
                          }
                          return {
                            ...msg,
                            contents: [...msg.contents, newContent],
                          }
                        }
                        return msg
                      })
                      return { messages: updatedMessages }
                    })
                  },
                  toolResult: (data: ToolResultData) => {
                    // Add tool result content to the streaming-temp assistant message
                    set(state => {
                      const updatedMessages = state.messages.map(msg => {
                        if (msg.id === 'streaming-temp') {
                          const newContent: MessageContentItem = {
                            id: data.message_content_id,
                            message_id: data.message_id,
                            content_type: 'tool_result',
                            content: {
                              call_id: data.call_id,
                              result: data.result,
                              success: data.success,
                              error_message: data.error_message,
                            } as MessageContentDataToolResult,
                            sequence_order: msg.contents.length,
                            created_at: new Date().toISOString(),
                            updated_at: new Date().toISOString(),
                          }
                          return {
                            ...msg,
                            contents: [...msg.contents, newContent],
                          }
                        }
                        return msg
                      })
                      return { messages: updatedMessages }
                    })
                  },
                  titleUpdated: (data: TitleUpdatedData) => {
                    // Update conversation title in the chat store
                    set(state => {
                      if (state.conversation) {
                        return {
                          conversation: {
                            ...state.conversation,
                            title: data.title,
                          },
                        }
                      }
                      return {}
                    })

                    // Also update in conversations list
                    updateConversationTitle(conversationId, data.title)
                  },
                  complete: (_data: CompleteData) => {
                    // Handle completion events - replace streaming-temp with actual message
                    set(state => {
                      const streamingMsg = state.messages.find(
                        msg => msg.id === 'streaming-temp',
                      )
                      const finalMessage = streamingMsg
                        ? {
                            ...streamingMsg,
                            id: actualAssistantMessageId || assistantMessage.id,
                            contents: streamingMsg.contents.map(c => ({
                              ...c,
                              message_id:
                                actualAssistantMessageId || assistantMessage.id,
                            })),
                            updated_at: new Date().toISOString(),
                          }
                        : assistantMessage

                      return {
                        isStreaming: false,
                        sending: false,
                        messages: [
                          ...state.messages.filter(
                            msg => msg.id !== 'streaming-temp',
                          ),
                          finalMessage,
                        ],
                      }
                    })
                  },
                  error: _data => {
                    set({
                      error: 'Edit streaming failed',
                      sending: false,
                      isStreaming: false,
                      // Remove the streaming placeholder
                      messages: get().messages.filter(
                        (msg: Message) => msg.id !== 'streaming-temp',
                      ),
                    })
                  },
                  default: (event, data) => {
                    console.log(
                      'Unknown chat edit stream SSE event:',
                      event,
                      data,
                    )
                  },
                },
              },
            )
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
