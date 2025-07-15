import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../api/client'
import {
  CreateModelProviderRequest,
  ModelCapabilities,
  ModelProvider,
  ModelProviderModel,
} from '../types/api/modelProvider'
import {
  uploadFile,
  uploadFilesConcurrent,
  UploadProgress,
} from '../api/fileUpload'
import { getAuthToken, getBaseUrl } from '../api/core'

// Type alias for compatibility
type Model = ModelProviderModel

export interface FileUploadProgress {
  filename: string
  progress: number
  status: 'pending' | 'uploading' | 'completed' | 'error'
  error?: string
  size?: number
}

export interface ProcessedFile {
  temp_file_id: string
  filename: string
  file_type: string
  size_bytes: number
  checksum: string
  validation_issues: string[]
  is_main_file: boolean
}

export interface UploadSession {
  session_id: string
  files: ProcessedFile[]
  total_size_bytes: number
  main_filename: string
  provider_id: string
}

interface ModelProvidersState {
  // Data
  providers: ModelProvider[]
  modelsByProvider: Record<string, ModelProviderModel[]> // Store models by provider ID

  // Loading states
  loading: boolean
  creating: boolean
  updating: boolean
  deleting: boolean
  testingProxy: boolean
  loadingModels: Record<string, boolean> // Track loading state for provider models
  modelOperations: Record<string, boolean> // Track loading state for individual models

  // Upload states
  uploading: boolean
  uploadProgress: FileUploadProgress[]
  overallUploadProgress: number

  // Upload session state
  uploadSession: UploadSession | null

  // Internal state for upload control
  _uploadXhr?: XMLHttpRequest | null

  // Error state
  error: string | null

  // Actions
  loadProviders: () => Promise<void>
  createProvider: (data: CreateModelProviderRequest) => Promise<ModelProvider>
  updateProvider: (
    id: string,
    data: Partial<ModelProvider>,
  ) => Promise<ModelProvider>
  deleteProvider: (id: string) => Promise<void>
  cloneProvider: (id: string, name: string) => Promise<ModelProvider>

  // Model actions
  loadModels: (providerId: string) => Promise<void>
  addModel: (providerId: string, data: Partial<Model>) => Promise<Model>
  updateModel: (modelId: string, data: Partial<Model>) => Promise<Model>
  deleteModel: (modelId: string) => Promise<void>
  startModel: (modelId: string) => Promise<void> // For Candle
  stopModel: (modelId: string) => Promise<void> // For Candle
  enableModel: (modelId: string) => Promise<void>
  disableModel: (modelId: string) => Promise<void>

  // Upload model actions (for Candle) - New multi-step workflow
  uploadMultipleFiles: (
    providerId: string,
    files: File[],
    mainFilename: string,
  ) => Promise<UploadSession>
  commitUploadedFiles: (
    sessionId: string,
    providerId: string,
    name: string,
    alias: string,
    description: string | undefined,
    architecture: string,
    fileFormat: string,
    capabilities: ModelCapabilities,
    selectedFileIds: string[],
  ) => Promise<void>

  // Legacy upload actions (deprecated)
  createUploadModel: (
    providerId: string,
    name: string,
    alias: string,
    description?: string,
    architecture?: string,
    fileFormat?: string,
    metadata?: any,
  ) => Promise<{ id: string }>
  uploadModelFile: (modelId: string, file: File) => Promise<void>
  uploadModelFiles: (
    modelId: string,
    files: File[],
    mainFilename: string,
  ) => Promise<void>

  // Upload progress actions
  clearUploadProgress: () => void
  clearUploadSession: () => void
  cancelUpload: () => void

  // Proxy actions
  testProxy: (providerId: string) => Promise<boolean>

  // Utility actions
  clearError: () => void
  getProviderById: (id: string) => ModelProvider | undefined
  getModelById: (id: string) => Model | undefined
}

export const useModelProvidersStore = create<ModelProvidersState>()(
  subscribeWithSelector((set, get) => ({
    // Initial state
    providers: [],
    modelsByProvider: {},
    loading: false,
    creating: false,
    updating: false,
    deleting: false,
    testingProxy: false,
    loadingModels: {},
    modelOperations: {},
    uploading: false,
    uploadProgress: [],
    overallUploadProgress: 0,
    uploadSession: null,
    error: null,

    loadProviders: async () => {
      try {
        set({ loading: true, error: null })

        const response = await ApiClient.ModelProviders.list({
          page: 1,
          per_page: 50,
        })

        set({
          providers: response.providers,
          loading: false,
        })
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to load model providers',
          loading: false,
        })
        throw error
      }
    },

    createProvider: async (data: CreateModelProviderRequest) => {
      try {
        set({ creating: true, error: null })

        const provider = await ApiClient.ModelProviders.create(data)

        set(state => ({
          providers: [...state.providers, provider],
          creating: false,
        }))

        return provider
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to create provider',
          creating: false,
        })
        throw error
      }
    },

    updateProvider: async (id: string, data: Partial<ModelProvider>) => {
      try {
        set({ updating: true, error: null })

        const provider = await ApiClient.ModelProviders.update({
          provider_id: id,
          ...data,
        })

        set(state => ({
          providers: state.providers.map(p => (p.id === id ? provider : p)),
          updating: false,
        }))

        return provider
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to update provider',
          updating: false,
        })
        throw error
      }
    },

    deleteProvider: async (id: string) => {
      try {
        set({ deleting: true, error: null })

        await ApiClient.ModelProviders.delete({ provider_id: id })

        set(state => ({
          providers: state.providers.filter(p => p.id !== id),
          deleting: false,
        }))
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to delete provider',
          deleting: false,
        })
        throw error
      }
    },

    cloneProvider: async (id: string, name: string) => {
      try {
        set({ creating: true, error: null })

        const provider = await ApiClient.ModelProviders.clone({
          provider_id: id,
          name: name,
        } as any)

        set(state => ({
          providers: [...state.providers, provider],
          creating: false,
        }))

        return provider
      } catch (error) {
        set({
          error:
            error instanceof Error ? error.message : 'Failed to clone provider',
          creating: false,
        })
        throw error
      }
    },

    loadModels: async (providerId: string) => {
      try {
        set(state => ({
          loadingModels: { ...state.loadingModels, [providerId]: true },
          error: null,
        }))

        const models = await ApiClient.ModelProviders.listModels({
          provider_id: providerId,
        })

        set(state => ({
          modelsByProvider: { ...state.modelsByProvider, [providerId]: models },
          loadingModels: { ...state.loadingModels, [providerId]: false },
        }))
      } catch (error) {
        set(state => ({
          error:
            error instanceof Error ? error.message : 'Failed to load models',
          loadingModels: { ...state.loadingModels, [providerId]: false },
        }))
        throw error
      }
    },

    addModel: async (providerId: string, data: Partial<Model>) => {
      try {
        set({ creating: true, error: null })

        const model = await ApiClient.ModelProviders.addModel({
          provider_id: providerId,
          ...data,
        } as any)

        set(state => ({
          modelsByProvider: {
            ...state.modelsByProvider,
            [providerId]: [
              ...(state.modelsByProvider[providerId] || []),
              model,
            ],
          },
          creating: false,
        }))

        return model
      } catch (error) {
        set({
          error: error instanceof Error ? error.message : 'Failed to add model',
          creating: false,
        })
        throw error
      }
    },

    updateModel: async (modelId: string, data: Partial<Model>) => {
      try {
        set({ updating: true, error: null })

        const model = await ApiClient.Models.update({
          model_id: modelId,
          ...data,
        })

        set(state => ({
          modelsByProvider: Object.keys(state.modelsByProvider).reduce(
            (acc, providerId) => {
              acc[providerId] = state.modelsByProvider[providerId].map(m =>
                m.id === modelId ? model : m,
              )
              return acc
            },
            {} as Record<string, ModelProviderModel[]>,
          ),
          updating: false,
        }))

        return model
      } catch (error) {
        set({
          error:
            error instanceof Error ? error.message : 'Failed to update model',
          updating: false,
        })
        throw error
      }
    },

    deleteModel: async (modelId: string) => {
      try {
        set({ deleting: true, error: null })

        await ApiClient.Models.delete({ model_id: modelId })

        set(state => ({
          modelsByProvider: Object.keys(state.modelsByProvider).reduce(
            (acc, providerId) => {
              acc[providerId] = state.modelsByProvider[providerId].filter(
                m => m.id !== modelId,
              )
              return acc
            },
            {} as Record<string, ModelProviderModel[]>,
          ),
          deleting: false,
        }))
      } catch (error) {
        set({
          error:
            error instanceof Error ? error.message : 'Failed to delete model',
          deleting: false,
        })
        throw error
      }
    },

    startModel: async (modelId: string) => {
      try {
        set(state => ({
          modelOperations: { ...state.modelOperations, [modelId]: true },
          error: null,
        }))

        await ApiClient.Models.start({ model_id: modelId })

        // Update model status to starting
        set(state => ({
          modelsByProvider: Object.keys(state.modelsByProvider).reduce(
            (acc, providerId) => {
              acc[providerId] = state.modelsByProvider[providerId].map(m =>
                m.id === modelId ? { ...m, is_active: true } : m,
              )
              return acc
            },
            {} as Record<string, ModelProviderModel[]>,
          ),
          modelOperations: { ...state.modelOperations, [modelId]: false },
        }))
      } catch (error) {
        set(state => ({
          error:
            error instanceof Error ? error.message : 'Failed to start model',
          modelOperations: { ...state.modelOperations, [modelId]: false },
        }))
        throw error
      }
    },

    stopModel: async (modelId: string) => {
      try {
        set(state => ({
          modelOperations: { ...state.modelOperations, [modelId]: true },
          error: null,
        }))

        await ApiClient.Models.stop({ model_id: modelId })

        // Update model status to stopping
        set(state => ({
          modelsByProvider: Object.keys(state.modelsByProvider).reduce(
            (acc, providerId) => {
              acc[providerId] = state.modelsByProvider[providerId].map(m =>
                m.id === modelId ? { ...m, is_active: false } : m,
              )
              return acc
            },
            {} as Record<string, ModelProviderModel[]>,
          ),
          modelOperations: { ...state.modelOperations, [modelId]: false },
        }))
      } catch (error) {
        set(state => ({
          error:
            error instanceof Error ? error.message : 'Failed to stop model',
          modelOperations: { ...state.modelOperations, [modelId]: false },
        }))
        throw error
      }
    },

    enableModel: async (modelId: string) => {
      try {
        set({ updating: true, error: null })

        await ApiClient.Models.enable({ model_id: modelId })

        // Update model status to enabled
        set(state => ({
          modelsByProvider: Object.keys(state.modelsByProvider).reduce(
            (acc, providerId) => {
              acc[providerId] = state.modelsByProvider[providerId].map(m =>
                m.id === modelId ? { ...m, enabled: true } : m,
              )
              return acc
            },
            {} as Record<string, ModelProviderModel[]>,
          ),
          updating: false,
        }))
      } catch (error) {
        set({
          error:
            error instanceof Error ? error.message : 'Failed to enable model',
          updating: false,
        })
        throw error
      }
    },

    disableModel: async (modelId: string) => {
      try {
        set({ updating: true, error: null })

        await ApiClient.Models.disable({ model_id: modelId })

        // Update model status to disabled
        set(state => ({
          modelsByProvider: Object.keys(state.modelsByProvider).reduce(
            (acc, providerId) => {
              acc[providerId] = state.modelsByProvider[providerId].map(m =>
                m.id === modelId ? { ...m, enabled: false } : m,
              )
              return acc
            },
            {} as Record<string, ModelProviderModel[]>,
          ),
          updating: false,
        }))
      } catch (error) {
        set({
          error:
            error instanceof Error ? error.message : 'Failed to disable model',
          updating: false,
        })
        throw error
      }
    },

    testProxy: async (providerId: string) => {
      try {
        set({ testingProxy: true, error: null })

        const result = await ApiClient.ModelProviders.testProxy({
          provider_id: providerId,
        } as any)

        set({ testingProxy: false })

        return result.success
      } catch (error) {
        set({
          error:
            error instanceof Error ? error.message : 'Failed to test proxy',
          testingProxy: false,
        })
        throw error
      }
    },

    // New multi-step upload workflow
    uploadMultipleFiles: async (
      providerId: string,
      files: File[],
      mainFilename: string,
    ): Promise<UploadSession> => {
      try {
        set({
          uploading: true,
          error: null,
          uploadProgress: files.map(file => ({
            filename: file.name,
            progress: 0,
            status: 'pending' as const,
            size: file.size,
          })),
          overallUploadProgress: 0,
          uploadSession: null,
        })

        // Create multipart form data
        const formData = new FormData()

        // Add provider_id and main_filename
        formData.append('provider_id', providerId)
        formData.append('main_filename', mainFilename)

        // Add all files and calculate total size
        let totalSize = 0
        const fileSizes: number[] = []
        files.forEach((file, index) => {
          formData.append('files', file)
          fileSizes[index] = file.size
          totalSize += file.size
        })

        // Upload files to backend with simulated per-file progress
        const baseUrl = await getBaseUrl()
        const uploadSession: UploadSession = await new Promise(
          (resolve, reject) => {
            const xhr = new XMLHttpRequest()

            // Store xhr for cancellation
            set(state => ({ ...state, _uploadXhr: xhr }))

            // Track overall upload progress and simulate individual file progress
            xhr.upload.addEventListener('progress', event => {
              if (event.lengthComputable) {
                const bytesUploaded = event.loaded
                const overallProgress = Math.round(
                  (bytesUploaded / totalSize) * 100,
                )

                // Calculate which files have been uploaded based on bytes
                let accumulatedBytes = 0
                const fileProgresses: number[] = []

                for (let i = 0; i < files.length; i++) {
                  const fileStartBytes = accumulatedBytes
                  const fileEndBytes = accumulatedBytes + fileSizes[i]

                  if (bytesUploaded >= fileEndBytes) {
                    // File is fully uploaded
                    fileProgresses[i] = 100
                  } else if (bytesUploaded > fileStartBytes) {
                    // File is partially uploaded
                    const fileProgress =
                      ((bytesUploaded - fileStartBytes) / fileSizes[i]) * 100
                    fileProgresses[i] = Math.round(fileProgress)
                  } else {
                    // File hasn't started uploading yet
                    fileProgresses[i] = 0
                  }

                  accumulatedBytes += fileSizes[i]
                }

                // Update individual file progress
                set(state => ({
                  uploadProgress: state.uploadProgress.map((f, idx) => ({
                    ...f,
                    progress: fileProgresses[idx] || 0,
                    status:
                      fileProgresses[idx] === 100
                        ? ('completed' as const)
                        : fileProgresses[idx] > 0
                          ? ('uploading' as const)
                          : ('pending' as const),
                  })),
                  overallUploadProgress: overallProgress,
                }))
              }
            })

            // Handle completion
            xhr.addEventListener('load', () => {
              if (xhr.status >= 200 && xhr.status < 300) {
                try {
                  const response = JSON.parse(xhr.responseText)

                  // Mark all files as completed
                  set(state => ({
                    uploadProgress: state.uploadProgress.map(f => ({
                      ...f,
                      progress: 100,
                      status: 'completed' as const,
                    })),
                    overallUploadProgress: 100,
                  }))

                  resolve(response)
                } catch {
                  reject(new Error('Failed to parse response'))
                }
              } else {
                // Try to get detailed error from response
                let errorMessage = `Upload failed: ${xhr.status} ${xhr.statusText}`
                try {
                  const errorData = JSON.parse(xhr.responseText)
                  if (errorData.error) {
                    errorMessage = errorData.error
                  }
                } catch {
                  // If we can't parse the error response, use the status text
                }
                reject(new Error(errorMessage))
              }
            })

            // Handle errors
            xhr.addEventListener('error', () => {
              reject(new Error('Upload failed: Network error'))
            })

            // Handle timeout
            xhr.addEventListener('timeout', () => {
              reject(new Error('Upload failed: Timeout'))
            })

            // Handle abort (cancellation)
            xhr.addEventListener('abort', () => {
              reject(new Error('Upload cancelled'))
            })

            // Setup and send request
            xhr.open(
              'POST',
              `${baseUrl}/api/admin/uploaded-models/upload-multipart`,
            )
            xhr.setRequestHeader('Authorization', `Bearer ${getAuthToken()}`)
            xhr.timeout = 300000 // 5 minute timeout for large files
            xhr.send(formData)
          },
        )

        // Update final state
        set({
          uploading: false,
          uploadSession,
          _uploadXhr: null, // Clear xhr reference
        })

        return uploadSession
      } catch (error) {
        set({
          error:
            error instanceof Error ? error.message : 'Failed to upload files',
          uploading: false,
          uploadProgress: files.map(file => ({
            filename: file.name,
            progress: 0,
            status: 'error' as const,
            error: error instanceof Error ? error.message : 'Upload failed',
            size: file.size,
          })),
          _uploadXhr: null, // Clear xhr reference
        })
        throw error
      }
    },

    commitUploadedFiles: async (
      sessionId: string,
      providerId: string,
      name: string,
      alias: string,
      description: string | undefined,
      architecture: string,
      fileFormat: string,
      capabilities: ModelCapabilities,
      selectedFileIds: string[],
    ): Promise<void> => {
      try {
        set({ creating: true, error: null })

        const newModel = await ApiClient.ModelUploads.commitUpload({
          session_id: sessionId,
          provider_id: providerId,
          name,
          alias,
          description,
          architecture,
          file_format: fileFormat,
          capabilities,
          selected_files: selectedFileIds,
        })

        // Update models state to include the new model
        set(state => ({
          modelsByProvider: {
            ...state.modelsByProvider,
            [providerId]: [
              ...(state.modelsByProvider[providerId] || []),
              newModel,
            ],
          },
          creating: false,
          uploadSession: null, // Clear the session after successful commit
        }))

        // Reload providers to get the latest state
        await get().loadProviders()
      } catch (error) {
        set({
          error:
            error instanceof Error ? error.message : 'Failed to commit upload',
          creating: false,
        })
        throw error
      }
    },

    createUploadModel: async (
      providerId: string,
      name: string,
      alias: string,
      description?: string,
      architecture?: string,
      fileFormat?: string,
      metadata?: any,
    ) => {
      try {
        set({ creating: true, error: null })

        const response = await ApiClient.ModelUploads.create({
          provider_id: providerId,
          name,
          alias,
          description,
          architecture: architecture || 'llama', // Default to llama if not specified
          file_format: fileFormat,
          metadata: metadata,
        })

        set({ creating: false })

        return { id: response.id }
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to create upload model',
          creating: false,
        })
        throw error
      }
    },

    uploadModelFile: async (modelId: string, file: File) => {
      try {
        set({
          uploading: true,
          error: null,
          uploadProgress: [
            {
              filename: file.name,
              progress: 0,
              status: 'pending',
              size: file.size,
            },
          ],
          overallUploadProgress: 0,
        })

        await uploadFile(modelId, file, file.name, {
          onProgress: (progress: UploadProgress) => {
            set(state => ({
              uploadProgress: state.uploadProgress.map(f =>
                f.filename === file.name
                  ? { ...f, progress: progress.percentage, status: 'uploading' }
                  : f,
              ),
              overallUploadProgress: progress.percentage,
            }))
          },
          onComplete: () => {
            set(state => ({
              uploadProgress: state.uploadProgress.map(f =>
                f.filename === file.name
                  ? { ...f, progress: 100, status: 'completed' }
                  : f,
              ),
              overallUploadProgress: 100,
              uploading: false,
            }))
          },
          onError: (error: string) => {
            set(state => ({
              uploadProgress: state.uploadProgress.map(f =>
                f.filename === file.name ? { ...f, status: 'error', error } : f,
              ),
              uploading: false,
              error,
            }))
          },
        })
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to upload model file',
          uploading: false,
        })
        throw error
      }
    },

    uploadModelFiles: async (
      modelId: string,
      files: File[],
      mainFilename: string,
    ) => {
      try {
        // Initialize upload progress for all files
        const initialProgress: FileUploadProgress[] = files.map(file => ({
          filename: file.name,
          progress: 0,
          status: 'pending' as const,
          size: file.size,
        }))

        set({
          uploading: true,
          error: null,
          uploadProgress: initialProgress,
          overallUploadProgress: 0,
        })

        let uploadAborted = false

        // Store abort controller for cancellation
        const abortController = new AbortController()
        set(state => ({ ...state, _abortController: abortController }))

        await uploadFilesConcurrent(modelId, files, mainFilename, 3, {
          onFileProgress: (fileIndex: number, progress: UploadProgress) => {
            if (uploadAborted) return

            set(state => ({
              uploadProgress: state.uploadProgress.map((f, i) =>
                i === fileIndex
                  ? { ...f, progress: progress.percentage, status: 'uploading' }
                  : f,
              ),
            }))
          },
          onFileComplete: (fileIndex: number) => {
            if (uploadAborted) return

            set(state => ({
              uploadProgress: state.uploadProgress.map((f, i) =>
                i === fileIndex
                  ? { ...f, progress: 100, status: 'completed' }
                  : f,
              ),
            }))
          },
          onFileError: (fileIndex: number, error: string) => {
            if (uploadAborted) return

            set(state => ({
              uploadProgress: state.uploadProgress.map((f, i) =>
                i === fileIndex ? { ...f, status: 'error', error } : f,
              ),
            }))
          },
          onOverallProgress: (completedFiles: number, totalFiles: number) => {
            if (uploadAborted) return

            const overallProgress = Math.round(
              (completedFiles / totalFiles) * 100,
            )
            set({ overallUploadProgress: overallProgress })
          },
          onAllComplete: () => {
            if (!uploadAborted) {
              set({ uploading: false, overallUploadProgress: 100 })
            }
          },
        })
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to upload model files',
          uploading: false,
        })
        throw error
      }
    },

    clearUploadProgress: () => {
      set({
        uploadProgress: [],
        overallUploadProgress: 0,
        uploading: false,
      })
    },

    clearUploadSession: () => {
      set({ uploadSession: null })
    },

    cancelUpload: () => {
      set(state => {
        // Abort XMLHttpRequest upload
        const uploadXhr = (state as any)._uploadXhr
        if (uploadXhr) {
          uploadXhr.abort()
        }

        // Also handle legacy AbortController for backward compatibility
        const abortController = (state as any)._abortController
        if (abortController) {
          abortController.abort()
        }

        return {
          uploading: false,
          uploadProgress: state.uploadProgress.map(f =>
            f.status === 'uploading' || f.status === 'pending'
              ? { ...f, status: 'error', error: 'Upload cancelled' }
              : f,
          ),
          _uploadXhr: null, // Clear the xhr reference
        }
      })
    },

    clearError: () => {
      set({ error: null })
    },

    getProviderById: (id: string) => {
      return get().providers.find(p => p.id === id)
    },

    getModelById: (id: string) => {
      const { modelsByProvider } = get()
      for (const providerId of Object.keys(modelsByProvider)) {
        const model = modelsByProvider[providerId].find(m => m.id === id)
        if (model) return model
      }
      return undefined
    },
  })),
)
