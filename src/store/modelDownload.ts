import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../api/client'
import { Model, ModelCapabilities, ModelSettings } from '../types/api/model'

export interface DownloadProgress {
  phase: string
  current: number
  total: number
  message: string
}

export interface DownloadFromRepositoryRequest {
  provider_id: string
  repository_id: string
  repository_path: string
  main_filename: string
  repository_branch?: string
  name: string
  alias: string
  description?: string
  file_format: string
  capabilities?: ModelCapabilities
  settings?: ModelSettings
}

export interface DownloadInstance {
  id: string
  request: DownloadFromRepositoryRequest
  downloading: boolean
  progress: DownloadProgress | null
  error: string | null
  startedAt: Date
  completedAt?: Date
}

interface ModelDownloadState {
  // Download instances map
  downloads: Record<string, DownloadInstance>

  // Modal state
  modalOpen: boolean
  modalProviderId: string | null
  modalProviderType: string | null
  modalViewMode: boolean
  modalViewDownloadId: string | null
}

export const useModelDownloadStore = create<ModelDownloadState>()(
  subscribeWithSelector(
    (): ModelDownloadState => ({
      // Initial state
      downloads: {},

      // Modal initial state
      modalOpen: false,
      modalProviderId: null,
      modalProviderType: null,
      modalViewMode: false,
      modalViewDownloadId: null,
    }),
  ),
)

// Download model from repository with SSE progress tracking
export const downloadModelFromRepository = async (
  request: DownloadFromRepositoryRequest,
): Promise<{ model: Model; downloadId: string }> => {
  // Generate a unique ID for this download
  const downloadId = `${Date.now()}-${Math.random().toString(36).substring(2, 11)}`

  // Create initial download instance
  const downloadInstance: DownloadInstance = {
    id: downloadId,
    request,
    downloading: true,
    progress: {
      phase: 'Starting',
      current: 0,
      total: 100,
      message: 'Initializing repository download...',
    },
    error: null,
    startedAt: new Date(),
  }

  // Add to downloads map
  useModelDownloadStore.setState(state => ({
    downloads: {
      ...state.downloads,
      [downloadId]: downloadInstance,
    },
  }))

  try {
    // biome-ignore lint/suspicious/noAsyncPromiseExecutor: needed for SSE handling
    const model = await new Promise<Model>(async (resolve, reject) => {
      let isRejected = false
      await ApiClient.Admin.downloadFromRepository(request, {
        SSE: (event: string, data: any) => {
          if (event === 'progress') {
            useModelDownloadStore.setState(state => ({
              downloads: {
                ...state.downloads,
                [downloadId]: {
                  ...state.downloads[downloadId],
                  progress: {
                    phase: data.phase,
                    current: data.current,
                    total: data.total,
                    message: data.message || 'Downloading...',
                  },
                },
              },
            }))
          } else if (event === 'complete') {
            useModelDownloadStore.setState(state => ({
              downloads: {
                ...state.downloads,
                [downloadId]: {
                  ...state.downloads[downloadId],
                  downloading: false,
                  progress: null,
                  completedAt: new Date(),
                },
              },
            }))

            const model = data.model as Model
            resolve(model)
          } else if (event === 'error') {
            useModelDownloadStore.setState(state => ({
              downloads: {
                ...state.downloads,
                [downloadId]: {
                  ...state.downloads[downloadId],
                  downloading: false,
                  progress: null,
                  error: data.message || 'Download failed',
                  completedAt: new Date(),
                },
              },
            }))
            !isRejected && reject(new Error(data.message || 'Download failed'))
            isRejected = true
          }
        },
      }).catch(e => {
        console.error('Download error:', e)
        !isRejected && reject(e)
        isRejected = true
      })
    })

    return { model, downloadId }
  } catch (error) {
    useModelDownloadStore.setState(state => ({
      downloads: {
        ...state.downloads,
        [downloadId]: {
          ...state.downloads[downloadId],
          downloading: false,
          progress: null,
          error:
            error instanceof Error
              ? error.message
              : 'Failed to download from repository',
          completedAt: new Date(),
        },
      },
    }))
    throw error
  }
}

export const cancelModelDownload = (downloadId: string): void => {
  useModelDownloadStore.setState(state => ({
    downloads: {
      ...state.downloads,
      [downloadId]: {
        ...state.downloads[downloadId],
        downloading: false,
        progress: null,
        error: 'Download cancelled',
        completedAt: new Date(),
      },
    },
  }))
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
  return Object.values(state.downloads).filter(download => download.downloading)
}

export const findDownloadById = (
  downloadId: string,
): DownloadInstance | undefined => {
  const state = useModelDownloadStore.getState()
  return state.downloads[downloadId]
}

// Modal actions
export const openAddModelModal = (
  providerId: string,
  providerType: string,
): void => {
  useModelDownloadStore.setState({
    modalOpen: true,
    modalProviderId: providerId,
    modalProviderType: providerType,
    modalViewMode: false,
    modalViewDownloadId: null,
  })
}

export const openViewDownloadModal = (
  downloadId: string,
  providerType: string,
): void => {
  useModelDownloadStore.setState({
    modalOpen: true,
    modalProviderId: null, // Not needed for view mode
    modalProviderType: providerType,
    modalViewMode: true,
    modalViewDownloadId: downloadId,
  })
}

export const closeDownloadModal = (): void => {
  useModelDownloadStore.setState({
    modalOpen: false,
    modalProviderId: null,
    modalProviderType: null,
    modalViewMode: false,
    modalViewDownloadId: null,
  })
}
