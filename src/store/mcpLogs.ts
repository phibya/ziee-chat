import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../api/client'
import type { MCPLogEntry, MCPLogType } from '../types/api'
import { createStoreProxy } from '../utils/createStoreProxy.ts'
import { StoreApi, UseBoundStore } from 'zustand/index'
import { useEffect, useMemo, useRef } from 'react'
import { debounce } from '../utils/debounce'

interface MCPServerLogsState {
  // Server logs
  logs: MCPLogEntry[]

  // Connection state
  connection: {
    loading: boolean
    connected: boolean
    error: string | null
  }

  // UI preferences
  preferences: {
    selectedLogTypes: MCPLogType[]
    autoScroll: boolean
  }

  // Internal flags
  isSubscribing: boolean

  // Store management
  destroy: () => void

  // Actions
  subscribeToLogs: () => Promise<void>
  disconnectFromLogs: () => void
  clearLogs: () => void
  updatePreferences: (
    preferences: Partial<{
      selectedLogTypes: MCPLogType[]
      autoScroll: boolean
    }>,
  ) => void
  getFilteredLogs: () => MCPLogEntry[]
  reset: () => void
}

// Store map to keep the proxies
const MCPLogsStoreMap = new Map<
  string,
  ReturnType<
    typeof createStoreProxy<UseBoundStore<StoreApi<MCPServerLogsState>>>
  >
>()

// Map to track cleanup debounce functions for each server
const CleanupDebounceMap = new Map<string, ReturnType<typeof debounce>>()

// SSE Management (per server)
const sseAbortControllers: Record<string, AbortController | null> = {}
const intentionalDisconnections: Record<string, boolean> = {}

export const createMCPLogsStore = (serverId: string) => {
  if (MCPLogsStoreMap.has(serverId)) {
    return MCPLogsStoreMap.get(serverId)!
  }

  const store = create<MCPServerLogsState>()(
    subscribeWithSelector(
      (set, get): MCPServerLogsState => ({
        // Initial state
        logs: [],
        connection: {
          loading: false,
          connected: false,
          error: null,
        },
        preferences: {
          selectedLogTypes: ['Exec', 'In', 'Out', 'Err'] as MCPLogType[],
          autoScroll: true,
        },
        isSubscribing: false,

        destroy: () => {
          // Clean up SSE connection
          get().disconnectFromLogs()

          // Remove the store from the map and let the browser GC it
          MCPLogsStoreMap.delete(serverId)
        },

        // Actions
        subscribeToLogs: async () => {
          const state = get()
          const { connection, isSubscribing } = state

          // If already connected or currently subscribing, don't create another connection
          if (connection.connected || isSubscribing) {
            return
          }

          // Set subscribing flag to prevent duplicate calls
          set(prev => ({ ...prev, isSubscribing: true }))

          // Clean up any existing AbortController for this server
          if (sseAbortControllers[serverId]) {
            sseAbortControllers[serverId]?.abort()
            sseAbortControllers[serverId] = null
            // Small delay to ensure cleanup is complete
            await new Promise(resolve => setTimeout(resolve, 100))
          }

          try {
            console.log(
              `Establishing SSE connection for MCP server logs: ${serverId}`,
            )

            // Reset disconnection flag
            intentionalDisconnections[serverId] = false

            // Update connection state
            set(prev => ({
              ...prev,
              connection: {
                loading: true,
                connected: false,
                error: null,
              },
            }))

            await ApiClient.Mcp.streamServerLogs(
              { server_id: serverId },
              {
                SSE: {
                  __init: data => {
                    sseAbortControllers[serverId] = data.abortController
                    console.log(
                      `MCP logs SSE AbortController initialized for server ${serverId}`,
                    )

                    set(prev => ({
                      ...prev,
                      connection: {
                        ...prev.connection,
                        connected: true,
                      },
                      isSubscribing: false,
                    }))
                  },
                  logEntry: data => {
                    const currentLogs = get().logs

                    // Limit logs to last 1000 entries to prevent memory issues
                    const updatedLogs = [...currentLogs, data]
                    if (updatedLogs.length > 1000) {
                      updatedLogs.splice(0, updatedLogs.length - 1000)
                    }

                    set(prev => ({
                      ...prev,
                      logs: updatedLogs,
                      connection: {
                        ...prev.connection,
                        loading: false,
                        error: null,
                      },
                    }))
                  },
                  connected: data => {
                    console.log(
                      `Connected to MCP log stream for server: ${data.server_name}`,
                    )

                    set(prev => ({
                      ...prev,
                      connection: {
                        ...prev.connection,
                        loading: false,
                        error: null,
                      },
                    }))
                  },
                  initialLogsComplete: data => {
                    console.log(
                      `Initial logs loaded for server ${serverId}:`,
                      data.timestamp,
                    )
                  },
                  error: data => {
                    console.error(
                      `MCP logs SSE error for server ${serverId}:`,
                      data.error,
                    )

                    set(prev => ({
                      ...prev,
                      connection: {
                        ...prev.connection,
                        error: data.error,
                      },
                    }))
                  },
                  default: (event, data) => {
                    console.log(
                      `Unknown MCP log SSE event for server ${serverId}:`,
                      event,
                      data,
                    )
                  },
                },
              },
            )
          } catch (error) {
            // Ignore AbortErrors as they are expected during cleanup/disconnection
            if (error instanceof Error && error.name === 'AbortError') {
              if (intentionalDisconnections[serverId]) {
                console.log(
                  `MCP logs SSE connection for server ${serverId} was intentionally aborted during cleanup`,
                )
              } else {
                console.log(
                  `MCP logs SSE connection for server ${serverId} was aborted (unexpected)`,
                )
              }

              set(prev => ({
                ...prev,
                connection: {
                  loading: false,
                  connected: false,
                  error: null,
                },
                isSubscribing: false,
              }))
              return
            }

            console.error(
              `Failed to establish MCP logs SSE connection for server ${serverId}:`,
              error,
            )

            set(prev => ({
              ...prev,
              connection: {
                loading: false,
                connected: false,
                error:
                  error instanceof Error ? error.message : 'Failed to connect',
              },
              isSubscribing: false,
            }))
          }
        },

        disconnectFromLogs: () => {
          console.log(
            `Disconnecting MCP logs monitoring for server ${serverId}`,
          )

          // Set flag to indicate intentional disconnection
          intentionalDisconnections[serverId] = true

          // Abort the SSE connection if AbortController is available
          if (sseAbortControllers[serverId]) {
            sseAbortControllers[serverId]?.abort()
            sseAbortControllers[serverId] = null
            console.log(
              `MCP logs SSE connection aborted for server ${serverId}`,
            )
          }

          set(prev => ({
            ...prev,
            connection: {
              loading: false,
              connected: false,
              error: null,
            },
            logs: [],
            isSubscribing: false,
          }))
        },

        clearLogs: () => {
          set(prev => ({
            ...prev,
            logs: [],
          }))
        },

        updatePreferences: preferences => {
          set(prev => ({
            ...prev,
            preferences: {
              ...prev.preferences,
              ...preferences,
            },
          }))
        },

        getFilteredLogs: () => {
          const { logs, preferences } = get()

          if (
            !preferences?.selectedLogTypes ||
            preferences.selectedLogTypes.length === 0
          ) {
            return logs
          }

          return logs.filter(log =>
            preferences.selectedLogTypes.includes(log.log_type),
          )
        },

        reset: () => {
          set({
            logs: [],
            connection: {
              loading: false,
              connected: false,
              error: null,
            },
            preferences: {
              selectedLogTypes: ['Exec', 'In', 'Out', 'Err'] as MCPLogType[],
              autoScroll: true,
            },
            isSubscribing: false,
          })
        },
      }),
    ),
  )

  const storeProxy = createStoreProxy(store)
  MCPLogsStoreMap.set(serverId, storeProxy)

  return storeProxy
}

// Hook for components to use server-specific logs stores
export const useMCPLogsStore = (serverId?: string) => {
  const currentId = serverId
  const prevIdRef = useRef<string | undefined>(currentId)

  useEffect(() => {
    const prevId = prevIdRef.current

    // If serverId changed, set up debounced cleanup for the previous one
    if (prevId && prevId !== currentId) {
      // Create debounced cleanup function for the previous server
      const cleanupFn = debounce(
        () => {
          const store = MCPLogsStoreMap.get(prevId)
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
    prevIdRef.current = currentId
  }, [currentId])

  return useMemo(() => {
    const id = currentId
    if (!id) {
      // Return a default store for cases where there's no server ID
      return createMCPLogsStore('default')
    }

    // Cancel any existing debounced cleanup for this server since it's being accessed again
    const existingCleanup = CleanupDebounceMap.get(id)
    if (existingCleanup) {
      existingCleanup.cancel()
      CleanupDebounceMap.delete(id)
    }

    return createMCPLogsStore(id)
  }, [serverId])
}

export const disconnectFromServerLogs = (serverId: string): void => {
  const store = MCPLogsStoreMap.get(serverId)
  if (store) {
    store.disconnectFromLogs()
  }
}

export const clearServerLogs = (serverId: string): void => {
  const store = MCPLogsStoreMap.get(serverId)
  if (store) {
    store.clearLogs()
  }
}

export const updateServerPreferences = (
  serverId: string,
  preferences: Partial<{ selectedLogTypes: MCPLogType[]; autoScroll: boolean }>,
): void => {
  const store = MCPLogsStoreMap.get(serverId)
  if (store) {
    store.updatePreferences(preferences)
  }
}
