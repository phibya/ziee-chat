import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'

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

let eventSource: EventSource | null = null
const MAX_LOG_LINES = 1000 // Keep last 1000 lines in memory

export const connectToApiProxyLogs = async () => {
  if (eventSource) {
    eventSource.close()
  }

  useApiProxyLogMonitorStore.setState({
    connecting: true,
    error: null,
  })

  try {
    const token = localStorage.getItem('authToken') // Adjust based on your auth implementation
    const url = `/api/admin/api-proxy-server/logs/stream${token ? `?token=${token}` : ''}`

    eventSource = new EventSource(url)

    eventSource.onopen = () => {
      useApiProxyLogMonitorStore.setState({
        connected: true,
        connecting: false,
        error: null,
      })
    }

    eventSource.addEventListener('connected', event => {
      console.log('API proxy log monitor connected:', event.data)
    })

    eventSource.addEventListener('log_update', event => {
      try {
        const data = JSON.parse(event.data)
        if (data.lines && Array.isArray(data.lines)) {
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
          })
        }
      } catch (error) {
        console.error('Failed to parse log message:', error)
      }
    })

    eventSource.onerror = error => {
      console.error('EventSource error:', error)
      useApiProxyLogMonitorStore.setState({
        connected: false,
        connecting: false,
        error: 'Connection to log stream failed',
      })
    }
  } catch (error) {
    console.error('Failed to connect to logs:', error)
    useApiProxyLogMonitorStore.setState({
      connecting: false,
      error: error instanceof Error ? error.message : 'Unknown error',
    })
    throw error
  }
}

export const disconnectFromApiProxyLogs = () => {
  if (eventSource) {
    eventSource.close()
    eventSource = null
  }

  useApiProxyLogMonitorStore.setState({
    connected: false,
    connecting: false,
    error: null,
  })
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
