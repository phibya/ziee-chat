import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { MessageBranch } from '../types/api/chat'
import { ApiClient } from '../api/client'
import { createStoreProxy } from '../utils/createStoreProxy'
import { StoreApi, UseBoundStore } from 'zustand/index'
import { useMemo, useRef, useEffect } from 'react'
import { debounce } from '../utils/debounce'

export interface MessageBranchState {
  // Store branches for this specific message ID
  branches: MessageBranch[]
  messageId: string | null
  originatedId: string | null

  // Loading states
  loading: boolean

  // Error state
  error: string | null

  // Store management
  destroy: () => void

  // Actions
  loadBranches: () => Promise<void>
  clearError: () => void
  reset: () => void
}

// Store map to keep the proxies by message ID
const MessageBranchStoreMap = new Map<
  string,
  ReturnType<
    typeof createStoreProxy<UseBoundStore<StoreApi<MessageBranchState>>>
  >
>()

// Map to track cleanup debounce functions for each message ID
const CleanupDebounceMap = new Map<string, ReturnType<typeof debounce>>()

export const createMessageBranchesStore = (
  messageId: string,
  originatedId: string,
) => {
  if (MessageBranchStoreMap.has(messageId)) {
    return MessageBranchStoreMap.get(messageId)!
  }

  const store = create<MessageBranchState>()(
    subscribeWithSelector(
      (set, get): MessageBranchState => ({
        // Initial state
        branches: [],
        messageId: messageId,
        originatedId: originatedId,
        loading: false,
        error: null,

        destroy: () => {
          // Remove the store from the map and let the browser GC it
          MessageBranchStoreMap.delete(messageId)
        },

        // Actions
        loadBranches: async () => {
          if (get().loading) {
            // If already loading, do nothing
            return
          }
          try {
            set({ loading: true, error: null })

            const branches = await ApiClient.Chat.getMessageBranches({
              message_id: messageId,
            })

            set({
              branches: branches,
              loading: false,
            })
          } catch (error) {
            set({
              error:
                error instanceof Error
                  ? error.message
                  : 'Failed to load message branches',
              loading: false,
            })
            throw error
          }
        },

        clearError: () => {
          set({ error: null })
        },

        reset: () => {
          set({
            branches: [],
            loading: false,
            error: null,
          })
        },
      }),
    ),
  )

  const storeProxy = createStoreProxy(store)
  MessageBranchStoreMap.set(messageId, storeProxy)
  return storeProxy
}

// Hook for components to use message-specific branch stores
export const useMessageBranchStore = (
  messageId: string,
  originatedId: string,
) => {
  const prevIdRef = useRef<string | undefined>(messageId)

  useEffect(() => {
    const prevId = prevIdRef.current

    // If messageId changed, set up debounced cleanup for the previous one
    if (prevId && prevId !== messageId) {
      // Create debounced cleanup function for the previous message
      const cleanupFn = debounce(
        () => {
          const store = MessageBranchStoreMap.get(prevId)
          if (store) {
            store.destroy()
          }
          CleanupDebounceMap.delete(prevId)
        },
        5 * 60 * 1000,
      ) // 5 minutes

      CleanupDebounceMap.set(prevId, cleanupFn)
      cleanupFn()
    }

    // Update the ref for the next render
    prevIdRef.current = messageId
  }, [messageId])

  return useMemo(() => {
    const id = messageId
    if (!id || !originatedId) {
      // Return a default store for cases where there's no message ID
      return createMessageBranchesStore('default', 'default')
    }

    // Cancel any existing debounced cleanup for this message since it's being accessed again
    const existingCleanup = CleanupDebounceMap.get(id)
    if (existingCleanup) {
      existingCleanup.cancel()
      CleanupDebounceMap.delete(id)
    }

    return createMessageBranchesStore(id, originatedId)
  }, [messageId])
}

export const removeMessageBranchStoreByOriginatedId = (
  originatedMessageId: string,
): void => {
  Array.from(MessageBranchStoreMap.values()).forEach(store => {
    store.__state.originatedId === originatedMessageId &&
      store.__state.destroy()
  })
}
