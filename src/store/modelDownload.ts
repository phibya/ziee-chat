import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../api/client'
import type {
  DownloadFromRepositoryRequest,
  DownloadInstance,
  DownloadProgressUpdate,
} from '../types/api/modelDownloads.ts'

interface ModelDownloadState {
  // Download instances map
  downloads: Record<string, DownloadInstance>
  // SSE connection state
  sseConnected: boolean
  sseError: string | null
  // Reconnection attempt count
  reconnectAttempts: number
}

export const useModelDownloadStore = create<ModelDownloadState>()(
  subscribeWithSelector(
    (): ModelDownloadState => ({
      // Initial state
      downloads: {},
      sseConnected: false,
      sseError: null,
      reconnectAttempts: 0,
    }),
  ),
)

// Download model from repository using new API
export const downloadModelFromRepository = async (
  request: DownloadFromRepositoryRequest,
  onStart?: (downloadId: string) => void,
): Promise<{ downloadId: string }> => {
  try {
    // Call the new initiate download endpoint that returns immediately
    const downloadInstance =
      await ApiClient.Admin.initiateRepositoryDownload(request)

    // Add to downloads map
    useModelDownloadStore.setState(state => ({
      downloads: {
        ...state.downloads,
        [downloadInstance.id]: downloadInstance,
      },
    }))

    // Call onStart callback with the download ID
    onStart?.(downloadInstance.id)

    // Set up download tracking subscription if not already done
    setupDownloadTracking()

    return { downloadId: downloadInstance.id }
  } catch (error) {
    console.error('Failed to initiate download:', error)
    throw error
  }
}

export const cancelModelDownload = async (
  downloadId: string,
): Promise<void> => {
  try {
    // Call backend to cancel the download
    await ApiClient.Admin.cancelDownload({ download_id: downloadId })

    // Remove from local state immediately since backend will delete it
    useModelDownloadStore.setState(state => {
      const { [downloadId]: _, ...remaining } = state.downloads
      return { downloads: remaining }
    })
  } catch (error) {
    console.error('Failed to cancel download:', error)
    throw error
  }
}

export const deleteModelDownload = async (
  downloadId: string,
): Promise<void> => {
  try {
    // Call backend to delete the download from database
    await ApiClient.Admin.deleteDownload({ download_id: downloadId })

    // Remove from local state
    useModelDownloadStore.setState(state => {
      const { [downloadId]: _, ...remaining } = state.downloads
      return { downloads: remaining }
    })
  } catch (error) {
    console.error('Failed to delete download:', error)
    throw error
  }
}

export const clearModelDownload = (downloadId: string): void => {
  useModelDownloadStore.setState(state => {
    const { [downloadId]: _, ...remaining } = state.downloads
    return { downloads: remaining }
  })
}

export const clearAllModelDownloads = (): void => {
  useModelDownloadStore.setState({ downloads: {} })
}

export const getAllActiveDownloads = (): DownloadInstance[] => {
  const state = useModelDownloadStore.getState()
  return Object.values(state.downloads).filter(
    download =>
      download.status === 'downloading' || download.status === 'pending',
  )
}

export const findDownloadById = (
  downloadId: string,
): DownloadInstance | undefined => {
  return useModelDownloadStore.getState().downloads[downloadId]
}

// SSE Subscription Management
let sseReconnectTimeout: ReturnType<typeof setTimeout> | null = null
const MAX_RECONNECT_ATTEMPTS = 5
const RECONNECT_DELAY = 3000

// Subscribe to download progress updates via SSE
export const subscribeToDownloadProgress = async (): Promise<void> => {
  const state = useModelDownloadStore.getState()

  // If already connected, don't create another connection
  if (state.sseConnected) {
    return
  }

  try {
    console.log('Establishing SSE connection for download progress')

    useModelDownloadStore.setState({
      sseConnected: true,
      sseError: null,
      reconnectAttempts: 0,
    })

    await ApiClient.Admin.subscribeDownloadProgress(undefined, {
      SSE: (event: string, data: any) => {
        try {
          switch (event) {
            case 'update':
              if (data.downloads) {
                // Update downloads in store with progress updates
                const currentDownloads =
                  useModelDownloadStore.getState().downloads
                const updatedDownloads: Record<string, DownloadInstance> = {
                  ...currentDownloads,
                }

                data.downloads.forEach(
                  (progressUpdate: DownloadProgressUpdate) => {
                    const existingDownload = updatedDownloads[progressUpdate.id]
                    if (existingDownload) {
                      // Merge progress update with existing download instance
                      updatedDownloads[progressUpdate.id] = {
                        ...existingDownload,
                        status: progressUpdate.status as any,
                        progress_data: {
                          phase: progressUpdate.phase || '',
                          current: progressUpdate.current || 0,
                          total: progressUpdate.total || 0,
                          message: progressUpdate.message || '',
                          download_speed: progressUpdate.speed_bps || 0,
                          eta_seconds: progressUpdate.eta_seconds || 0,
                        },
                        error_message: progressUpdate.error_message || null,
                        updated_at: new Date().toISOString(),
                      }
                    }
                  },
                )

                // Filter out cancelled and completed downloads before updating state
                const filteredDownloads: Record<string, DownloadInstance> = {}
                Object.entries(updatedDownloads).forEach(([id, download]) => {
                  if (
                    download.status !== 'cancelled' &&
                    download.status !== 'completed'
                  ) {
                    filteredDownloads[id] = download
                  }
                })

                useModelDownloadStore.setState({
                  downloads: filteredDownloads,
                })
              }
              break

            case 'complete':
              console.log('Downloads complete:', data.message)
              // Close the connection as no more downloads are active
              disconnectSSE()
              loadExistingDownloads()
              break

            case 'error':
              console.error('Download subscription error:', data.error)
              useModelDownloadStore.setState({
                sseError: data.error,
                sseConnected: false,
              })
              break

            default:
              console.log('Unknown SSE event:', event, data)
          }
        } catch (error) {
          console.error('Failed to handle SSE event:', error)
          useModelDownloadStore.setState({
            sseError:
              error instanceof Error
                ? error.message
                : 'Failed to handle SSE event',
            sseConnected: false,
          })
        }
      },
    })
  } catch (error) {
    console.error('Failed to establish SSE connection:', error)
    useModelDownloadStore.setState({
      sseConnected: false,
      sseError: error instanceof Error ? error.message : 'Failed to connect',
    })

    // Attempt reconnection if we have active downloads
    const activeDownloads = getAllActiveDownloads()
    if (activeDownloads.length > 0) {
      handleReconnection()
    }
  }
}

// Disconnect SSE connection
export const disconnectSSE = (): void => {
  useModelDownloadStore.setState({
    sseConnected: false,
    sseError: null,
  })

  // Clear any pending reconnection timeout
  if (sseReconnectTimeout) {
    clearTimeout(sseReconnectTimeout)
    sseReconnectTimeout = null
  }
}

// Handle reconnection logic
const handleReconnection = (): void => {
  const { reconnectAttempts } = useModelDownloadStore.getState()

  if (reconnectAttempts >= MAX_RECONNECT_ATTEMPTS) {
    console.error('Max reconnection attempts reached')
    useModelDownloadStore.setState({
      sseError: 'Failed to reconnect after multiple attempts',
    })
    return
  }

  // Clear existing timeout if any
  if (sseReconnectTimeout) {
    clearTimeout(sseReconnectTimeout)
  }

  // Increment reconnect attempts
  useModelDownloadStore.setState(state => ({
    reconnectAttempts: state.reconnectAttempts + 1,
  }))

  // Attempt reconnection after delay
  sseReconnectTimeout = setTimeout(async () => {
    console.log(
      `Attempting SSE reconnection (${reconnectAttempts + 1}/${MAX_RECONNECT_ATTEMPTS})`,
    )
    await subscribeToDownloadProgress()
  }, RECONNECT_DELAY)
}

// Load existing downloads from server
const loadExistingDownloads = async (): Promise<void> => {
  try {
    // Fetch all download instances from server
    const response = await ApiClient.Admin.listAllDownloads({})

    // Update store with existing downloads (exclude cancelled and completed)
    const downloads: Record<string, DownloadInstance> = {}
    response.downloads.forEach(download => {
      if (['pending', 'downloading', 'failed'].includes(download.status)) {
        downloads[download.id] = download
      }
    })

    useModelDownloadStore.setState({ downloads })

    console.log(
      `Loaded ${response.downloads.length} existing downloads from server`,
    )
  } catch (error) {
    console.error('Failed to load existing downloads:', error)
  }
}

// Set up download tracking subscription (called automatically when store changes)
let isSubscriptionSetup = false
const setupDownloadTracking = (): void => {
  if (isSubscriptionSetup) return
  isSubscriptionSetup = true

  // Subscribe to store changes to manage SSE connection
  useModelDownloadStore.subscribe(
    state => state.downloads,
    downloads => {
      const activeDownloads = Object.values(downloads).filter(
        d => d.status === 'downloading' || d.status === 'pending',
      )

      if (
        activeDownloads.length > 0 &&
        !useModelDownloadStore.getState().sseConnected
      ) {
        // We have active downloads but no SSE connection, establish one
        void subscribeToDownloadProgress()
      } else if (
        activeDownloads.length === 0 &&
        useModelDownloadStore.getState().sseConnected
      ) {
        // No active downloads and SSE is connected, disconnect
        disconnectSSE()
      }
    },
  )
}

// Initialize download tracking after authentication with provider read permission
export const initializeDownloadTracking = async (): Promise<void> => {
  try {
    // Set up the subscription tracking
    setupDownloadTracking()

    // Load existing downloads from server
    await loadExistingDownloads()
  } catch (error) {
    console.error('Failed to initialize download tracking:', error)
  }
}
