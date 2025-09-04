import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../api/client'
import { createRAGInstanceStore } from './ragInstance'
import type {
  SSERAGInstanceStatusConnectedData,
  SSERAGInstanceStatusUpdateData,
  SSERAGInstanceStatusErrorData,
} from '../types/api'

interface RAGStatusState {
  // Instance status data (using existing generated types)
  currentStatus: SSERAGInstanceStatusUpdateData | null

  // Connection state
  statusLoading: boolean
  statusError: string | null
  sseConnected: boolean
  sseError: string | null
  sseAbortController: AbortController | null
}

export const useRAGStatusStore = create<RAGStatusState>()(
  subscribeWithSelector(
    (): RAGStatusState => ({
      currentStatus: null,
      statusLoading: false,
      statusError: null,
      sseConnected: false,
      sseError: null,
      sseAbortController: null,
    }),
  ),
)

// Store methods using existing API
export const subscribeToRAGStatus = async (
  instanceId: string,
): Promise<void> => {
  const state = useRAGStatusStore.getState()

  if (state.sseConnected) {
    return
  }

  // Clean up existing connection
  if (state.sseAbortController) {
    state.sseAbortController.abort()
    await new Promise(resolve => setTimeout(resolve, 100))
  }

  try {
    useRAGStatusStore.setState({
      sseError: null,
      statusLoading: true,
      sseAbortController: null,
    })

    await ApiClient.Rag.subscribeInstanceStatus(
      { instance_id: instanceId, include_files: true },
      {
        SSE: {
          __init: data => {
            useRAGStatusStore.setState({
              sseConnected: true,
              sseAbortController: data.abortController,
            })
          },
          connected: (_data: SSERAGInstanceStatusConnectedData) => {
            console.log('RAG status monitoring connected')
            useRAGStatusStore.setState({
              statusLoading: false,
              sseError: null,
            })
          },
          update: (data: SSERAGInstanceStatusUpdateData) => {
            useRAGStatusStore.setState({
              currentStatus: data,
              statusLoading: false,
              statusError: null,
            })

            // Update the ragInstance store with the latest is_active status
            const ragInstanceStore = createRAGInstanceStore(data.instance_id)
            if (ragInstanceStore.__state.ragInstance) {
              ragInstanceStore.__setState({
                ragInstance: {
                  ...ragInstanceStore.__state.ragInstance,
                  is_active: data.is_active,
                },
              })
            }
          },
          error: (data: SSERAGInstanceStatusErrorData) => {
            console.error('RAG status SSE error:', data.error)
            useRAGStatusStore.setState({
              sseError: data.error,
              statusLoading: false,
            })
          },
          default: (event, data) => {
            console.log('Unknown RAG status SSE event:', event, data)
          },
        },
      },
    )
  } catch (error) {
    if (error instanceof Error && error.name === 'AbortError') {
      console.log('RAG status SSE connection was aborted')
      useRAGStatusStore.setState({
        sseConnected: false,
        statusLoading: false,
      })
      return
    }

    console.error('Failed to establish RAG status SSE connection:', error)
    useRAGStatusStore.setState({
      sseConnected: false,
      sseError: error instanceof Error ? error.message : 'Failed to connect',
      statusLoading: false,
    })
  }
}

export const disconnectRAGStatus = (): void => {
  console.log('Disconnecting RAG status monitoring')

  const state = useRAGStatusStore.getState()

  if (state.sseAbortController) {
    state.sseAbortController.abort()
  }

  useRAGStatusStore.setState({
    sseConnected: false,
    sseError: null,
    currentStatus: null,
    statusLoading: false,
    sseAbortController: null,
  })
}

export const clearRAGStatusError = (): void => {
  useRAGStatusStore.setState({
    statusError: null,
    sseError: null,
  })
}
