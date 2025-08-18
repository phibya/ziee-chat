import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../../api/client'
import type {
  HardwareInfo,
  HardwareUsageUpdate,
  HardwareInfoResponse,
} from '../../types'

interface HardwareState {
  // Static hardware information
  hardwareInfo: HardwareInfo | null
  hardwareLoading: boolean
  hardwareError: string | null
  hardwareInitialized: boolean

  // Real-time usage data
  currentUsage: HardwareUsageUpdate | null
  usageLoading: boolean
  usageError: string | null

  // SSE connection state
  sseConnected: boolean
  sseError: string | null
}

export const useHardwareStore = create<HardwareState>()(
  subscribeWithSelector(
    (): HardwareState => ({
      // Static hardware info
      hardwareInfo: null,
      hardwareLoading: false,
      hardwareError: null,
      hardwareInitialized: false,

      // Real-time usage data
      currentUsage: null,
      usageLoading: false,
      usageError: null,

      // SSE connection state
      sseConnected: false,
      sseError: null,
    }),
  ),
)

// Load static hardware information
export const loadHardwareInfo = async (): Promise<void> => {
  useHardwareStore.setState({ hardwareLoading: true, hardwareError: null })

  try {
    const response: HardwareInfoResponse =
      await ApiClient.Admin.getHardwareInfo()
    useHardwareStore.setState({
      hardwareInfo: response.hardware,
      hardwareInitialized: true,
      hardwareLoading: false,
      hardwareError: null,
    })
  } catch (error) {
    console.error('Hardware info loading failed:', error)
    useHardwareStore.setState({
      hardwareLoading: false,
      hardwareError: error instanceof Error ? error.message : 'Unknown error',
      hardwareInitialized: false,
    })
    throw error
  }
}

// Clear hardware errors
export const clearHardwareError = (): void => {
  useHardwareStore.setState({
    hardwareError: null,
    usageError: null,
    sseError: null,
  })
}

// SSE Subscription Management for real-time usage monitoring
let sseAbortController: AbortController | null = null
let isIntentionallyDisconnecting = false

// Subscribe to hardware usage updates via SSE
export const subscribeToHardwareUsage = async (): Promise<void> => {
  const state = useHardwareStore.getState()

  // If already connected, don't create another connection
  if (state.sseConnected) {
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
    console.log('Establishing SSE connection for hardware usage monitoring')

    // Reset disconnection flag
    isIntentionallyDisconnecting = false

    useHardwareStore.setState({
      sseError: null,
      usageLoading: true,
    })

    await ApiClient.Admin.subscribeHardwareUsage(undefined, {
      SSE: (event: string, data: any) => {
        try {
          switch (event) {
            case '__init':
              // Store the AbortController for later use
              if (data?.abortController) {
                sseAbortController = data.abortController
                console.log('Hardware SSE AbortController initialized')
                // Set connected status once AbortController is ready
                useHardwareStore.setState({
                  sseConnected: true,
                })
              }
              break

            case 'update':
              if (data) {
                // Update current usage data
                useHardwareStore.setState({
                  currentUsage: data as HardwareUsageUpdate,
                  usageLoading: false,
                  usageError: null,
                })
              }
              break

            case 'connected':
              console.log(
                'Hardware usage monitoring connected:',
                data?.message || 'Connected',
              )
              useHardwareStore.setState({
                usageLoading: false,
                sseError: null,
              })
              break

            case 'error':
              console.error('Hardware usage subscription error:', data?.error)
              useHardwareStore.setState({
                sseError: data?.error || 'Hardware monitoring error',
                sseConnected: false,
                usageLoading: false,
              })
              break

            default:
              console.log('Unknown hardware SSE event:', event, data)
          }
        } catch (error) {
          console.error('Failed to handle hardware SSE event:', error)
          useHardwareStore.setState({
            sseError:
              error instanceof Error
                ? error.message
                : 'Failed to handle SSE event',
            sseConnected: false,
            usageLoading: false,
          })
        }
      },
    })
  } catch (error) {
    // Ignore AbortErrors as they are expected during cleanup/disconnection
    if (error instanceof Error && error.name === 'AbortError') {
      if (isIntentionallyDisconnecting) {
        console.log(
          'Hardware SSE connection was intentionally aborted during cleanup',
        )
      } else {
        console.log('Hardware SSE connection was aborted (unexpected)')
      }
      useHardwareStore.setState({
        sseConnected: false,
        usageLoading: false,
      })
      return
    }

    console.error('Failed to establish hardware SSE connection:', error)
    useHardwareStore.setState({
      sseConnected: false,
      sseError: error instanceof Error ? error.message : 'Failed to connect',
      usageLoading: false,
    })
  }
}

// Disconnect hardware usage SSE connection
export const disconnectHardwareUsage = (): void => {
  console.log('Disconnecting hardware usage monitoring')

  // Set flag to indicate intentional disconnection
  isIntentionallyDisconnecting = true

  // Abort the SSE connection if AbortController is available
  if (sseAbortController) {
    sseAbortController.abort()
    sseAbortController = null
    console.log('Hardware SSE connection aborted')
  }

  useHardwareStore.setState({
    sseConnected: false,
    sseError: null,
    currentUsage: null,
    usageLoading: false,
  })

  // Reset flag after disconnection
  isIntentionallyDisconnecting = false
}
