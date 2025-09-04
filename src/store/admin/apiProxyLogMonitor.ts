import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../../api/client'

interface ApiProxyLogMonitorState {
  logs: string[]
  connected: boolean
  connecting: boolean
  error: string | null
  logCount: number
  lastUpdate: string | null
  autoScroll: boolean
}

export const useApiProxyLogMonitorStore = create<ApiProxyLogMonitorState>()(
  subscribeWithSelector((_set, _get) => ({
    logs: [],
    connected: false,
    connecting: false,
    error: null,
    logCount: 0,
    lastUpdate: null,
    autoScroll: true,
  })),
)

// SSE Subscription Management for log monitoring
let sseAbortController: AbortController | null = null
let isIntentionallyDisconnecting = false
const MAX_LOG_LINES = 1000 // Keep last 1000 lines in memory

// Subscribe to API proxy log updates via SSE
export const connectToApiProxyLogs = async (): Promise<void> => {
  const state = useApiProxyLogMonitorStore.getState()

  // If already connected, don't create another connection
  if (state.connected) {
    return
  }

  // Clean up any existing AbortController
  if (sseAbortController) {
    sseAbortController.abort()
    sseAbortController = null
    // Small delay to ensure cleanup is complete
    await new Promise(resolve => setTimeout(resolve, 100))
  }

  try {
    console.log('Establishing SSE connection for API proxy log monitoring')

    // Reset disconnection flag
    isIntentionallyDisconnecting = false

    useApiProxyLogMonitorStore.setState({
      error: null,
      connecting: true,
    })

    await ApiClient.Admin.subscribeApiProxyServerLogs(undefined, {
      SSE: {
        __init: data => {
          sseAbortController = data.abortController
          console.log('API Proxy Logs SSE AbortController initialized')
          useApiProxyLogMonitorStore.setState({
            connected: true,
          })
        },
        logUpdate: data => {
          if (data?.lines && Array.isArray(data.lines)) {
            const currentState = useApiProxyLogMonitorStore.getState()
            const newLogs = [...currentState.logs, ...data.lines]

            // Keep only the last MAX_LOG_LINES
            if (newLogs.length > MAX_LOG_LINES) {
              newLogs.splice(0, newLogs.length - MAX_LOG_LINES)
            }

            useApiProxyLogMonitorStore.setState({
              logs: newLogs,
              logCount: newLogs.length,
              lastUpdate: data.timestamp || new Date().toISOString(),
              connecting: false,
              error: null,
            })
          }
        },
        connected: data => {
          console.log(
            'API proxy log monitoring connected:',
            data?.message || 'Connected',
          )
          useApiProxyLogMonitorStore.setState({
            connecting: false,
            error: null,
          })
        },
        default: (event, data) => {
          console.log('Unknown API proxy log SSE event:', event, data)
        },
      },
    })
  } catch (error) {
    // Ignore AbortErrors as they are expected during cleanup/disconnection
    if (error instanceof Error && error.name === 'AbortError') {
      if (isIntentionallyDisconnecting) {
        console.log(
          'API proxy log SSE connection was intentionally aborted during cleanup',
        )
      } else {
        console.log('API proxy log SSE connection was aborted (unexpected)')
      }
      useApiProxyLogMonitorStore.setState({
        connected: false,
        connecting: false,
      })
      return
    }

    console.error('Failed to establish API proxy log SSE connection:', error)
    useApiProxyLogMonitorStore.setState({
      connected: false,
      error: error instanceof Error ? error.message : 'Failed to connect',
      connecting: false,
    })
  }
}

// Disconnect API proxy log SSE connection
export const disconnectFromApiProxyLogs = (): void => {
  console.log('Disconnecting API proxy log monitoring')

  // Set flag to indicate intentional disconnection
  isIntentionallyDisconnecting = true

  // Abort the SSE connection if AbortController is available
  if (sseAbortController) {
    sseAbortController.abort()
    sseAbortController = null
    console.log('API proxy log SSE connection aborted')
  }

  useApiProxyLogMonitorStore.setState({
    connected: false,
    error: null,
    connecting: false,
  })

  // Reset flag after disconnection
  isIntentionallyDisconnecting = false
}

export const clearLogBuffer = () => {
  useApiProxyLogMonitorStore.setState({
    logs: [],
    logCount: 0,
  })
}

export const setAutoScroll = (autoScroll: boolean) => {
  useApiProxyLogMonitorStore.setState({ autoScroll })
}

export const downloadLogs = async () => {
  const state = useApiProxyLogMonitorStore.getState()
  const logContent = state.logs.join('\n')
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-')
  const filename = `api-proxy-logs-${timestamp}.txt`

  const blob = new Blob([logContent], { type: 'text/plain' })
  const url = URL.createObjectURL(blob)

  const link = document.createElement('a')
  link.href = url
  link.download = filename
  document.body.appendChild(link)
  link.click()
  document.body.removeChild(link)
  URL.revokeObjectURL(url)
}
