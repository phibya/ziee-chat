// src/store/chat/sseHandlers.ts
import {
  Message,
  MessageContentItem,
  MessageContentDataText,
  MessageContentDataToolCall,
  MessageContentDataToolCallPendingApproval,
  MessageContentDataToolResult,
  MessageBranch,
  ConnectedData,
  CompleteData,
  MessageContentChunkData,
  NewUserMessageData,
  NewAssistantMessageData,
  NewMessageContentData,
  ToolCallData,
  ToolCallPendingApprovalData,
  ToolCallPendingApprovalCancelData,
  ToolResultData,
  TitleUpdatedData,
} from '../../types'
import { updateConversationTitle } from '../conversations'
import { removeMessageBranchStoreByOriginatedId } from '../messageBranches'

// ============================================
// Types for Handler Context
// ============================================

export interface SSEHandlerContext {
  // Store methods
  set: (fn: (state: any) => any) => void
  get: () => any

  // Conversation context
  conversationId: string
  activeBranchId: string | null

  // Branch cache reference
  BranchMessagesCacheMap: Map<string, Message[]>

  // Message ID resolvers - these return the current target message ID
  getTargetMessageId: () => string | null

  // Mutable refs to track server-assigned IDs
  actualUserMessageId: { current: string | null }
  actualAssistantMessageId: { current: string | null }

  // Temporary message refs (for ID updates)
  userMessage: { current: Message | null }
  assistantMessage: { current: Message | null }

  // Edit-specific context (optional)
  editMessageId?: string
}

// ============================================
// Single Factory Function - Returns ALL Handlers
// ============================================

/**
 * Create ALL SSE event handlers with shared logic
 *
 * This function returns a complete set of SSE handlers that can be used by
 * both sendMessage and editMessage. The context parameter determines the
 * specific behavior for each endpoint.
 *
 * **Key Principle**: Backend determines which events to send, frontend has
 * handlers ready for all possible events.
 */
export const createSSEHandlers = (context: SSEHandlerContext) => {
  const {
    set,
    conversationId,
    activeBranchId,
    BranchMessagesCacheMap,
    getTargetMessageId,
    actualUserMessageId,
    actualAssistantMessageId,
    userMessage,
    assistantMessage,
    editMessageId,
  } = context

  return {
    // ============================================
    // Connection Event
    // ============================================
    connected: (_data: ConnectedData) => {
      console.log('Chat stream connected')
    },

    // ============================================
    // Message Lifecycle Events
    // ============================================
    newUserMessage: (data: NewUserMessageData) => {
      // Server has created the user message, track the ID
      actualUserMessageId.current = data.message_id

      // If we created a temporary user message, update it with the server ID
      if (userMessage.current) {
        set(state => {
          const updatedMessages = state.messages.map((msg: Message) => {
            if (msg.id === userMessage.current!.id) {
              return {
                ...msg,
                id: actualUserMessageId.current!,
                contents: msg.contents.map((c: MessageContentItem) => ({
                  ...c,
                  message_id: actualUserMessageId.current!,
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

    newAssistantMessage: (data: NewAssistantMessageData) => {
      // Server has created the assistant message, track the ID
      actualAssistantMessageId.current = data.message_id

      // If we created a temporary assistant message, update it with the server ID
      if (assistantMessage.current) {
        set(state => {
          const updatedMessages = state.messages.map((msg: Message) => {
            if (msg.id === assistantMessage.current!.id) {
              return {
                ...msg,
                id: actualAssistantMessageId.current!,
                contents: msg.contents.map((c: MessageContentItem) => ({
                  ...c,
                  message_id: actualAssistantMessageId.current!,
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

    editedMessage: (data: Message) => {
      // Only used by editMessage endpoint
      const editedMessage = data
      set(state => ({
        messages: state.messages.map((msg: Message) =>
          msg.id === editMessageId ? editedMessage : msg,
        ),
      }))
      removeMessageBranchStoreByOriginatedId(editedMessage.originated_from_id)
    },

    createdBranch: (data: MessageBranch) => {
      // Only used by editMessage endpoint
      const newBranch = data
      set(_state => ({ activeBranchId: newBranch.id }))
    },

    // ============================================
    // Content Streaming Events
    // ============================================
    newMessageContent: (data: NewMessageContentData) => {
      set(state => {
        const targetMessageId = getTargetMessageId()
        if (!targetMessageId) return {}

        const updatedMessages = state.messages.map((msg: Message) => {
          if (msg.id === targetMessageId) {
            // Create new text content item
            const newContent: MessageContentItem = {
              id: data.message_content_id,
              message_id: data.message_id,
              content_type: 'text',
              content: { text: '' } as MessageContentDataText,
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

    messageContentChunk: (data: MessageContentChunkData) => {
      if (!data.delta) return

      set(state => {
        const targetMessageId = getTargetMessageId()
        if (!targetMessageId) return {}

        const updatedMessages = state.messages.map((msg: Message) => {
          if (msg.id === targetMessageId) {
            // Find the specific content item by message_content_id
            const contentExists = msg.contents.some(
              c => c.id === data.message_content_id,
            )
            if (contentExists) {
              const updatedContents = msg.contents.map(c => {
                if (
                  c.id === data.message_content_id &&
                  c.content_type === 'text'
                ) {
                  return {
                    ...c,
                    content: {
                      ...c.content,
                      text:
                        (c.content as MessageContentDataText).text + data.delta,
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
    },

    // ============================================
    // Tool Events
    // ============================================
    toolCall: (data: ToolCallData) => {
      set(state => {
        const targetMessageId = getTargetMessageId()
        if (!targetMessageId) return {}

        const updatedMessages = state.messages.map((msg: Message) => {
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

    toolCallPendingApproval: (data: ToolCallPendingApprovalData) => {
      set(state => {
        const targetMessageId = getTargetMessageId()
        if (!targetMessageId) return {}

        const updatedMessages = state.messages.map((msg: Message) => {
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

    toolCallPendingApprovalCancel: (
      data: ToolCallPendingApprovalCancelData,
    ) => {
      set(state => {
        const updatedMessages = state.messages.map((msg: Message) => {
          // Find and update the content with matching message_content_id
          const updatedContents = msg.contents.map(
            (content: MessageContentItem) => {
              if (
                content.id === data.message_content_id &&
                content.content_type === 'tool_call_pending_approval'
              ) {
                return {
                  ...content,
                  content: {
                    ...content.content,
                    is_approved: false,
                  } as MessageContentDataToolCallPendingApproval,
                  updated_at: new Date().toISOString(),
                }
              }
              return content
            },
          )

          // Only return updated message if contents changed
          if (updatedContents !== msg.contents) {
            return { ...msg, contents: updatedContents }
          }
          return msg
        })

        return { messages: updatedMessages }
      })
    },

    toolResult: (data: ToolResultData) => {
      set(state => {
        const targetMessageId = getTargetMessageId()
        if (!targetMessageId) return {}

        const updatedMessages = state.messages.map((msg: Message) => {
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

    // ============================================
    // Metadata Events
    // ============================================
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

    // ============================================
    // Completion Events
    // ============================================
    complete: (_data: CompleteData) => {
      const targetMessage = assistantMessage.current
      const actualId = actualAssistantMessageId.current

      // Handle temporary message ID replacement
      if (targetMessage) {
        set(state => {
          const updatedMessages = state.messages.map((msg: Message) => {
            if (msg.id === targetMessage.id) {
              return {
                ...msg,
                id: actualId || targetMessage.id,
                contents: msg.contents.map((c: MessageContentItem) => ({
                  ...c,
                  message_id: actualId || targetMessage.id,
                })),
                updated_at: new Date().toISOString(),
              } as Message
            }
            return msg
          })

          // Update cache when streaming is complete
          if (activeBranchId) {
            const cacheKey = `${conversationId}:${activeBranchId}`
            BranchMessagesCacheMap.set(cacheKey, updatedMessages)
          }

          return {
            isStreaming: false,
            sending: false,
            messages: updatedMessages,
          }
        })
      } else {
        // No temporary message, just update state
        set(_state => ({
          isStreaming: false,
          sending: false,
        }))
      }
    },

    error: (data: { error: string }) => {
      set(state => ({
        error: data.error,
        sending: false,
        isStreaming: false,
        // Clean up streaming-temp message if it exists (editMessage)
        messages: state.messages.filter(
          (msg: Message) => msg.id !== 'streaming-temp',
        ),
      }))
      console.error('Streaming error:', data)
    },

    default: (event: string, data: any) => {
      console.log('Unknown SSE event:', event, data)
    },
  }
}
