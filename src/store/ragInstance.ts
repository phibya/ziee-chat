import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../api/client'
import { RAGInstance, RAGInstanceFile, RAGInstanceFilesListResponse, UpdateRAGInstanceRequest } from '../types/api'
import { createStoreProxy } from '../utils/createStoreProxy.ts'
import { StoreApi, UseBoundStore } from 'zustand/index'
import { useEffect, useMemo, useRef } from 'react'
import { useParams } from 'react-router-dom'
import { debounce } from '../utils/debounce'
import { updateRAGInstanceInList } from './rag.ts'

export interface FileUploadProgress {
  id: string
  filename: string
  progress: number
  status: 'pending' | 'uploading' | 'completed' | 'error'
  size: number
  error?: string
}

export interface RAGInstanceState {
  // Data
  ragInstance: RAGInstance | null
  files: RAGInstanceFile[]

  // Loading states
  loading: boolean
  updating: boolean
  uploading: boolean

  // File upload progress (current RAG instance only)
  uploadProgress: FileUploadProgress[]
  overallUploadProgress: number

  // File loading state (current RAG instance only)
  filesLoading: boolean
  filesError: string | null

  // Search and pagination state
  searchQuery: string
  currentPage: number
  filesPerPage: number
  totalFiles: number

  // Multi-select state
  selectedFiles: string[]
  bulkOperationInProgress: boolean

  // Error state
  error: string | null

  // Store management
  destroy: () => void

  // Actions
  loadRAGInstance: () => Promise<void>
  updateRAGInstance: (data: UpdateRAGInstanceRequest) => Promise<RAGInstance | undefined>
  loadFiles: (page?: number, search?: string) => Promise<void>
  nextPage: () => Promise<void>
  previousPage: () => Promise<void>
  goToPage: (page: number) => Promise<void>
  changePageSize: (pageSize: number) => Promise<void>
  uploadFiles: (files: globalThis.File[]) => Promise<RAGInstanceFile[]>
  cancelFileUpload: () => void
  removeUploadProgress: (progressId: string) => void
  clearFilesError: () => void
  clearError: () => void
  reset: () => void
}

// Store map to keep the proxies
const RAGInstanceStoreMap = new Map<
  string,
  ReturnType<typeof createStoreProxy<UseBoundStore<StoreApi<RAGInstanceState>>>>
>()

// Map to track cleanup debounce functions for each RAG instance
const CleanupDebounceMap = new Map<string, ReturnType<typeof debounce>>()

export const createRAGInstanceStore = (ragInstance: string | RAGInstance) => {
  let ragInstanceId: string
  if (typeof ragInstance === 'string') {
    ragInstanceId = ragInstance
  } else {
    ragInstanceId = ragInstance.id
  }

  if (RAGInstanceStoreMap.has(ragInstanceId)) {
    return RAGInstanceStoreMap.get(ragInstanceId)!
  }

  const store = create<RAGInstanceState>()(
    subscribeWithSelector(
      (set, get): RAGInstanceState => ({
        // Initial state
        ragInstance: typeof ragInstance === 'string' ? null : ragInstance,
        files: [],
        loading: false,
        updating: false,
        uploading: false,
        uploadProgress: [],
        overallUploadProgress: 0,
        filesLoading: false,
        filesError: null,
        searchQuery: '',
        currentPage: 1,
        filesPerPage: 10,
        totalFiles: 0,
        selectedFiles: [],
        bulkOperationInProgress: false,
        error: null,

        destroy: () => {
          // Remove the store from the map and let the browser GC it
          RAGInstanceStoreMap.delete(ragInstanceId)
        },

        // Actions
        loadRAGInstance: async () => {
          const state = get()
          if (state.loading) {
            return
          }
          try {
            set({ loading: true, error: null })

            const response = await ApiClient.Rag.getInstance({
              instance_id: ragInstanceId,
            })

            set({
              ragInstance: response,
              loading: false,
              // Clear upload state when switching RAG instances
              uploadProgress: [],
              uploading: false,
              overallUploadProgress: 0,
              filesError: null,
            })
          } catch (error) {
            set({
              error:
                error instanceof Error
                  ? error.message
                  : 'Failed to load RAG instance',
              loading: false,
            })
            throw error
          }
        },

        updateRAGInstance: async data => {
          const state = get()
          if (state.updating) {
            return
          }
          try {
            set({ updating: true, error: null })

            const ragInstance = await ApiClient.Rag.updateInstance({
              instance_id: ragInstanceId,
              ...data,
            })

            set({
              ragInstance: ragInstance,
              updating: false,
            })

            // Update the list store as well
            await updateRAGInstanceInList(ragInstanceId, data)

            return ragInstance
          } catch (error) {
            set({
              error:
                error instanceof Error
                  ? error.message
                  : 'Failed to update RAG instance',
              updating: false,
            })
            throw error
          }
        },

        loadFiles: async (page?: number, search?: string) => {
          const state = get()
          if (state.filesLoading) {
            return
          }
          try {
            set({ filesLoading: true, filesError: null })

            const currentPage = page || state.currentPage
            const searchQuery = search !== undefined ? search : state.searchQuery

            const response: RAGInstanceFilesListResponse = await ApiClient.Rag.listInstanceFiles({
              instance_id: ragInstanceId,
              page: currentPage,
              per_page: state.filesPerPage,
              search: searchQuery || undefined,
            })

            set({
              files: response.files || [],
              currentPage,
              searchQuery,
              totalFiles: response.total || 0,
              filesLoading: false,
            })
          } catch (error) {
            set({
              filesError:
                error instanceof Error
                  ? error.message
                  : 'Failed to load RAG instance files',
              filesLoading: false,
            })
            throw error
          }
        },

        nextPage: async () => {
          const state = get()
          const totalPages = Math.ceil(state.totalFiles / state.filesPerPage)
          const nextPage = Math.min(state.currentPage + 1, totalPages)
          if (nextPage !== state.currentPage) {
            await get().loadFiles(nextPage)
          }
        },

        previousPage: async () => {
          const state = get()
          const prevPage = Math.max(state.currentPage - 1, 1)
          if (prevPage !== state.currentPage) {
            await get().loadFiles(prevPage)
          }
        },

        goToPage: async (page: number) => {
          const state = get()
          const totalPages = Math.ceil(state.totalFiles / state.filesPerPage)
          const targetPage = Math.max(1, Math.min(page, totalPages))
          if (targetPage !== state.currentPage) {
            await get().loadFiles(targetPage)
          }
        },

        changePageSize: async (pageSize: number) => {
          const state = get()
          const newPageSize = Math.max(1, Math.min(pageSize, 100)) // Cap at 100
          if (newPageSize !== state.filesPerPage) {
            set({ filesPerPage: newPageSize })
            // Reset to page 1 when changing page size
            await get().loadFiles(1)
          }
        },

        uploadFiles: async files => {
          try {
            // Initialize upload progress with unique IDs (append to existing)
            const newFileProgress = files.map(file => ({
              id: crypto.randomUUID(),
              filename: file.name,
              progress: 0,
              status: 'pending' as const,
              size: file.size,
            }))

            set(state => ({
              uploading: true,
              uploadProgress: [...state.uploadProgress, ...newFileProgress],
              overallUploadProgress: 0,
              filesError: null,
            }))

            const uploadedFiles: RAGInstanceFile[] = []

            // Upload files sequentially to better track progress
            for (let i = 0; i < files.length; i++) {
              const file = files[i]
              const fileProgressId = newFileProgress[i].id

              // Update current file status to uploading
              set(state => ({
                uploadProgress: state.uploadProgress.map(
                  (fp: FileUploadProgress) =>
                    fp.id === fileProgressId
                      ? { ...fp, status: 'uploading' as const }
                      : fp,
                ),
              }))

              try {
                // Create FormData for the file
                const formData = new FormData()
                formData.append('file', file, file.name)
                formData.append('instance_id', ragInstanceId)

                // Call the upload API with progress tracking
                const response = await ApiClient.Rag.uploadInstanceFile(
                  formData as any, // FormData with instance_id
                  {
                    fileUploadProgress: {
                      onProgress: (progress: number) => {
                        // Update file-specific progress
                        set(state => ({
                          uploadProgress: state.uploadProgress.map(
                            (fp: FileUploadProgress) =>
                              fp.id === fileProgressId
                                ? { ...fp, progress: progress * 100 }
                                : fp,
                          ),
                          overallUploadProgress:
                            (i * 100 + progress * 100) / files.length,
                        }))
                      },
                      onComplete: () => {
                        // Remove completed file from upload progress
                        set(state => ({
                          uploadProgress: state.uploadProgress.filter(
                            (fp: FileUploadProgress) =>
                              fp.id !== fileProgressId,
                          ),
                        }))
                      },
                      onError: (error: string) => {
                        // Mark this file as failed
                        set(state => ({
                          uploadProgress: state.uploadProgress.map(
                            (fp: FileUploadProgress) =>
                              fp.id === fileProgressId
                                ? {
                                    ...fp,
                                    status: 'error' as const,
                                    error: error,
                                  }
                                : fp,
                          ),
                        }))
                      },
                    },
                  },
                )

                // The backend should have created the RAGInstanceFile entry
                // We'll reload the files list to get the proper RAGInstanceFile data
                // For now, just track that upload was successful
                console.log('File uploaded successfully:', response.file.filename)
              } catch (fileError) {
                // Mark this file as failed
                set(state => ({
                  uploadProgress: state.uploadProgress.map(
                    (fp: FileUploadProgress) =>
                      fp.id === fileProgressId
                        ? {
                            ...fp,
                            status: 'error' as const,
                            error:
                              fileError instanceof Error
                                ? fileError.message
                                : 'Upload failed',
                          }
                        : fp,
                  ),
                }))
              }
            }

            // Update overall progress to complete and reload files list
            set({
              overallUploadProgress: 100,
              uploading: false,
            })

            // Reload files from server to get the latest RAGInstanceFile entries
            await get().loadFiles()

            return uploadedFiles
          } catch (error) {
            set({
              filesError:
                error instanceof Error
                  ? error.message
                  : 'Failed to upload files',
              uploading: false,
              uploadProgress: [],
              overallUploadProgress: 0,
            })
            throw error
          }
        },

        cancelFileUpload: () => {
          set({
            uploading: false,
            uploadProgress: [],
            overallUploadProgress: 0,
          })
        },

        removeUploadProgress: (progressId: string) => {
          set(state => ({
            uploadProgress: state.uploadProgress.filter(
              (fp: FileUploadProgress) => fp.id !== progressId,
            ),
          }))
        },

        clearFilesError: () => {
          set({ filesError: null })
        },

        clearError: () => {
          set({ error: null })
        },

        reset: () => {
          set({
            ragInstance: null,
            files: [],
            loading: false,
            updating: false,
            uploading: false,
            uploadProgress: [],
            overallUploadProgress: 0,
            filesLoading: false,
            filesError: null,
            searchQuery: '',
            currentPage: 1,
            filesPerPage: 10,
            totalFiles: 0,
            selectedFiles: [],
            bulkOperationInProgress: false,
            error: null,
          })
        },
      }),
    ),
  )

  const storeProxy = createStoreProxy(store)
  RAGInstanceStoreMap.set(ragInstanceId, storeProxy)

  // Immediately load RAG instance when store is created (except for default store)
  if (ragInstanceId !== 'default') {
    storeProxy.__state.loadRAGInstance().catch(error => {
      console.error(`Failed to auto-load RAG instance ${ragInstanceId}:`, error)
    })
    storeProxy.__state.loadFiles().catch(error => {
      console.error(
        `Failed to auto-load files for RAG instance ${ragInstanceId}:`,
        error,
      )
    })
  }

  return storeProxy
}

// Store methods for file selection and bulk operations
export const toggleFileSelection = (instanceId: string, fileId: string) => {
  const store = RAGInstanceStoreMap.get(instanceId)
  if (!store) return
  
  const selected = store.__state.selectedFiles
  const newSelected = selected.includes(fileId) 
    ? selected.filter((id: string) => id !== fileId)
    : [...selected, fileId]
  store.__setState({ selectedFiles: newSelected })
}

export const selectAllVisibleFiles = (instanceId: string, fileIds: string[]) => {
  const store = RAGInstanceStoreMap.get(instanceId)
  if (!store) return
  store.__setState({ selectedFiles: fileIds })
}

export const clearFileSelection = (instanceId: string) => {
  const store = RAGInstanceStoreMap.get(instanceId)
  if (!store) return
  store.__setState({ selectedFiles: [] })
}

export const bulkDeleteFiles = async (instanceId: string, fileIds: string[]) => {
  const store = RAGInstanceStoreMap.get(instanceId)
  if (!store) return
  
  try {
    store.__setState({ bulkOperationInProgress: true })

    // Delete files sequentially using existing single delete API
    const deletePromises = fileIds.map(fileId => 
      ApiClient.Rag.deleteInstanceFile({ instance_id: instanceId, file_id: fileId })
    )

    await Promise.all(deletePromises)

    // Remove deleted files from local state
    const currentFiles = store.__state.files
    const remainingFiles = currentFiles.filter((file: RAGInstanceFile) => !fileIds.includes(file.file_id))
    
    store.__setState({
      files: remainingFiles,
      selectedFiles: [],
      bulkOperationInProgress: false,
    })

  } catch (error) {
    store.__setState({ bulkOperationInProgress: false })
    throw error
  }
}

export const searchFiles = async (instanceId: string, query: string) => {
  const store = RAGInstanceStoreMap.get(instanceId)
  if (!store) return
  
  // Reset to page 1 when searching
  await store.__state.loadFiles(1, query)
}

export const changePage = async (instanceId: string, page: number) => {
  const store = RAGInstanceStoreMap.get(instanceId)
  if (!store) return
  
  await store.__state.goToPage(page)
}

export const nextPage = async (instanceId: string) => {
  const store = RAGInstanceStoreMap.get(instanceId)
  if (!store) return
  
  await store.__state.nextPage()
}

export const previousPage = async (instanceId: string) => {
  const store = RAGInstanceStoreMap.get(instanceId)
  if (!store) return
  
  await store.__state.previousPage()
}

export const changePageSize = async (instanceId: string, pageSize: number) => {
  const store = RAGInstanceStoreMap.get(instanceId)
  if (!store) return
  
  await store.__state.changePageSize(pageSize)
}

// Hook for components to use RAG instance-specific stores
export const useRAGInstanceStore = (ragInstanceId?: string) => {
  const { ragInstanceId: paramRAGInstanceId } = useParams<{
    ragInstanceId?: string
  }>()
  const currentId = ragInstanceId || paramRAGInstanceId
  const prevIdRef = useRef<string | undefined>(currentId)

  useEffect(() => {
    const prevId = prevIdRef.current

    // If ragInstanceId changed, set up debounced cleanup for the previous one
    if (prevId && prevId !== currentId) {
      // Create debounced cleanup function for the previous RAG instance
      const cleanupFn = debounce(
        () => {
          const store = RAGInstanceStoreMap.get(prevId)
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
      // Return a default store for cases where there's no RAG instance ID
      return createRAGInstanceStore('default')
    }

    // Cancel any existing debounced cleanup for this RAG instance since it's being accessed again
    const existingCleanup = CleanupDebounceMap.get(id)
    if (existingCleanup) {
      existingCleanup.cancel()
      CleanupDebounceMap.delete(id)
    }

    return createRAGInstanceStore(id)
  }, [ragInstanceId, paramRAGInstanceId])
}