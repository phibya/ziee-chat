import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../api/client'
import { ConversationSummary } from '../types/api/chat'
import { createStoreProxy } from '../utils/createStoreProxy'
import { StoreApi, UseBoundStore } from 'zustand/index'
import { useMemo, useRef, useEffect } from 'react'
import { debounce } from '../utils/debounce'

export interface ChatHistoryState {
  // Data
  conversations: ConversationSummary[]
  searchResults: ConversationSummary[]
  projectId: string | null

  // Pagination state for list
  listCurrentPage: number
  listTotalPages: number
  listHasMore: boolean
  listTotal: number

  // Pagination state for search
  searchCurrentPage: number
  searchTotalPages: number
  searchHasMore: boolean
  searchTotal: number

  // Search state
  searchQuery: string
  isSearching: boolean

  // Selection state
  selectedConversations: Set<string>

  // Loading states
  loading: boolean
  loadingMore: boolean
  deleting: boolean

  // Error state
  error: string | null

  // Store management
  destroy: () => void

  // Actions
  loadConversationsList: (page?: number) => Promise<void>
  searchConversations: (query: string, page?: number) => Promise<void>
  loadNextListPage: () => Promise<void>
  loadNextSearchPage: () => Promise<void>
  deleteConversationById: (id: string) => Promise<void>
  updateConversationTitleById: (id: string, title: string) => Promise<void>
  clearSearchResults: () => void
  clearError: () => void
  reset: () => void

  // Selection actions
  selectConversation: (id: string) => void
  deselectConversation: (id: string) => void
  toggleConversationSelection: (id: string) => void
  selectAllConversations: () => void
  deselectAllConversations: () => void
  deleteSelectedConversations: () => Promise<void>
}

// Store map to keep the proxies by project ID (or 'global' for no project)
const ChatHistoryStoreMap = new Map<
  string,
  ReturnType<typeof createStoreProxy<UseBoundStore<StoreApi<ChatHistoryState>>>>
>()

// Map to track cleanup debounce functions for each project
const CleanupDebounceMap = new Map<string, ReturnType<typeof debounce>>()

export const createChatHistoryStore = (projectId?: string) => {
  const storeKey = projectId || 'global'

  if (ChatHistoryStoreMap.has(storeKey)) {
    return ChatHistoryStoreMap.get(storeKey)!
  }

  const store = create<ChatHistoryState>()(
    subscribeWithSelector(
      (set, get): ChatHistoryState => ({
        // Initial state
        conversations: [],
        searchResults: [],
        projectId: projectId || null,

        // Pagination state for list
        listCurrentPage: 0,
        listTotalPages: 0,
        listHasMore: false,
        listTotal: 0,

        // Pagination state for search
        searchCurrentPage: 0,
        searchTotalPages: 0,
        searchHasMore: false,
        searchTotal: 0,

        searchQuery: '',
        isSearching: false,
        selectedConversations: new Set(),
        loading: false,
        loadingMore: false,
        deleting: false,
        error: null,

        destroy: () => {
          // Remove the store from the map and let the browser GC it
          ChatHistoryStoreMap.delete(storeKey)
        },

        // Actions
        loadConversationsList: async (page = 1) => {
          const isFirstPage = page === 1
          const loadingState = isFirstPage ? 'loading' : 'loadingMore'

          if (get().loading || get().loadingMore) {
            // If already loading, do nothing
            return
          }
          try {
            set({
              [loadingState]: true,
              error: null,
            })

            const response = await ApiClient.Chat.listConversations({
              page,
              per_page: 20,
              project_id: projectId,
            })

            const totalPages = Math.ceil(response.total / response.per_page)
            const hasMore = response.page < totalPages

            set({
              conversations: isFirstPage
                ? response.conversations
                : [...get().conversations, ...response.conversations],
              listCurrentPage: response.page,
              listTotalPages: totalPages,
              listHasMore: hasMore,
              listTotal: response.total,
              [loadingState]: false,
            })
          } catch (error) {
            set({
              error:
                error instanceof Error
                  ? error.message
                  : 'Failed to load conversations',
              [loadingState]: false,
            })
            throw error
          }
        },

        searchConversations: async (query: string, page = 1) => {
          const isFirstPage = page === 1
          const loadingState = isFirstPage ? 'isSearching' : 'loadingMore'

          try {
            set({
              [loadingState]: true,
              searchQuery: query,
              error: null,
            })

            if (!query.trim()) {
              set({
                searchResults: [],
                isSearching: false,
                loadingMore: false,
                searchQuery: '',
                searchCurrentPage: 0,
                searchTotalPages: 0,
                searchHasMore: false,
              })
              return
            }

            const response = await ApiClient.Chat.searchConversations({
              q: query,
              page,
              per_page: 20,
              project_id: projectId,
            })

            const totalPages = Math.ceil(response.total / response.per_page)
            const hasMore = response.page < totalPages

            set({
              searchResults: isFirstPage
                ? response.conversations
                : [...get().searchResults, ...response.conversations],
              searchCurrentPage: response.page,
              searchTotalPages: totalPages,
              searchHasMore: hasMore,
              searchTotal: response.total,
              [loadingState]: false,
            })
          } catch (error) {
            set({
              error:
                error instanceof Error
                  ? error.message
                  : 'Failed to search conversations',
              [loadingState]: false,
            })
            throw error
          }
        },

        loadNextListPage: async () => {
          const state = get()
          if (!state.listHasMore || state.loading || state.loadingMore) {
            return
          }
          const nextPage = state.listCurrentPage + 1
          return get().loadConversationsList(nextPage)
        },

        loadNextSearchPage: async () => {
          const state = get()
          if (
            !state.searchHasMore ||
            state.isSearching ||
            state.loadingMore ||
            !state.searchQuery.trim()
          ) {
            return
          }
          const nextPage = state.searchCurrentPage + 1
          return get().searchConversations(state.searchQuery, nextPage)
        },

        deleteConversationById: async (id: string) => {
          try {
            set({ deleting: true, error: null })

            await ApiClient.Chat.deleteConversation({ conversation_id: id })

            set(state => {
              const newSelected = new Set(state.selectedConversations)
              newSelected.delete(id)

              return {
                conversations: state.conversations.filter(c => c.id !== id),
                searchResults: state.searchResults.filter(c => c.id !== id),
                selectedConversations: newSelected,
                deleting: false,
              }
            })
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

        updateConversationTitleById: async (id: string, title: string) => {
          try {
            set({ error: null })

            await ApiClient.Chat.updateConversation({
              conversation_id: id,
              title,
            })

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

        clearSearchResults: () => {
          set({
            searchQuery: '',
            searchResults: [],
            isSearching: false,
            searchCurrentPage: 0,
            searchTotalPages: 0,
            searchHasMore: false,
            searchTotal: 0,
          })
        },

        clearError: () => {
          set({ error: null })
        },

        reset: () => {
          set({
            conversations: [],
            searchResults: [],
            searchQuery: '',
            isSearching: false,
            selectedConversations: new Set(),
            loading: false,
            loadingMore: false,
            deleting: false,
            error: null,
            listCurrentPage: 0,
            listTotalPages: 0,
            listHasMore: false,
            listTotal: 0,
            searchCurrentPage: 0,
            searchTotalPages: 0,
            searchHasMore: false,
            searchTotal: 0,
          })
        },

        // Selection actions
        selectConversation: (id: string) => {
          const currentSelected = get().selectedConversations
          const newSelected = new Set(currentSelected)
          newSelected.add(id)
          set({ selectedConversations: newSelected })
        },

        deselectConversation: (id: string) => {
          const currentSelected = get().selectedConversations
          const newSelected = new Set(currentSelected)
          newSelected.delete(id)
          set({ selectedConversations: newSelected })
        },

        toggleConversationSelection: (id: string) => {
          const currentSelected = get().selectedConversations
          const newSelected = new Set(currentSelected)
          if (newSelected.has(id)) {
            newSelected.delete(id)
          } else {
            newSelected.add(id)
          }
          set({ selectedConversations: newSelected })
        },

        selectAllConversations: () => {
          const state = get()
          const currentList = state.searchQuery.trim()
            ? state.searchResults
            : state.conversations
          const allIds = new Set(currentList.map(conv => conv.id))
          set({ selectedConversations: allIds })
        },

        deselectAllConversations: () => {
          set({ selectedConversations: new Set() })
        },

        deleteSelectedConversations: async () => {
          const state = get()
          const selectedIds = Array.from(state.selectedConversations)

          if (selectedIds.length === 0) return

          try {
            set({ deleting: true, error: null })

            // Delete all selected conversations
            await Promise.all(
              selectedIds.map(id =>
                ApiClient.Chat.deleteConversation({ conversation_id: id }),
              ),
            )

            // Remove deleted conversations from both lists
            const updatedConversations = state.conversations.filter(
              c => !selectedIds.includes(c.id),
            )
            const updatedSearchResults = state.searchResults.filter(
              c => !selectedIds.includes(c.id),
            )

            set({
              conversations: updatedConversations,
              searchResults: updatedSearchResults,
              selectedConversations: new Set(),
              deleting: false,
            })
          } catch (error) {
            set({
              error:
                error instanceof Error
                  ? error.message
                  : 'Failed to delete conversations',
              deleting: false,
            })
            throw error
          }
        },
      }),
    ),
  )

  const storeProxy = createStoreProxy(store)
  ChatHistoryStoreMap.set(storeKey, storeProxy)

  storeProxy.__state.loadConversationsList()

  return storeProxy
}

// Hook for components to use project-specific chat history stores
export const useChatHistoryStore = (projectId?: string) => {
  const prevProjectIdRef = useRef<string | undefined>(projectId)

  useEffect(() => {
    const prevProjectId = prevProjectIdRef.current
    const storeKey = prevProjectId || 'global'

    // If projectId changed, set up debounced cleanup for the previous one
    if (prevProjectId !== projectId) {
      // Create debounced cleanup function for the previous project
      const cleanupFn = debounce(
        () => {
          const store = ChatHistoryStoreMap.get(storeKey)
          if (store) {
            store.destroy()
          }
          CleanupDebounceMap.delete(storeKey)
        },
        5 * 60 * 1000,
      ) // 5 minutes

      CleanupDebounceMap.set(storeKey, cleanupFn)
      cleanupFn()
    }

    // Update the ref for the next render
    prevProjectIdRef.current = projectId
  }, [projectId])

  return useMemo(() => {
    const storeKey = projectId || 'global'

    // Cancel any existing debounced cleanup for this project since it's being accessed again
    const existingCleanup = CleanupDebounceMap.get(storeKey)
    if (existingCleanup) {
      existingCleanup.cancel()
      CleanupDebounceMap.delete(storeKey)
    }

    return createChatHistoryStore(projectId)
  }, [projectId])
}
