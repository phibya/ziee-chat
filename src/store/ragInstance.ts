import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../api/client'
import { RAGInstance, RAGInstanceFile, UpdateRAGInstanceRequest } from '../types/api'
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

  // Error state
  error: string | null

  // Store management
  destroy: () => void

  // Actions
  loadRAGInstance: () => Promise<void>
  updateRAGInstance: (data: UpdateRAGInstanceRequest) => Promise<RAGInstance | undefined>
  loadFiles: () => Promise<void>
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

        loadFiles: async () => {
          const state = get()
          if (state.filesLoading) {
            return
          }
          try {
            set({ filesLoading: true, filesError: null })

            const response = await ApiClient.Rag.listInstanceFiles({
              instance_id: ragInstanceId,
            })

            set({
              files: response || [],
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
                // TODO: Implement file upload via API when available
                // For now, this is a placeholder that will be implemented in Phase 3
                console.log('File upload not yet implemented for RAG instance:', ragInstanceId, file.name)
                
                // Simulate upload progress
                for (let progress = 0; progress <= 100; progress += 10) {
                  set(state => ({
                    uploadProgress: state.uploadProgress.map(
                      (fp: FileUploadProgress) =>
                        fp.id === fileProgressId
                          ? { ...fp, progress }
                          : fp,
                    ),
                    overallUploadProgress:
                      (i * 100 + progress) / files.length,
                  }))
                  await new Promise(resolve => setTimeout(resolve, 100))
                }

                // Mark as completed and remove from progress
                set(state => ({
                  uploadProgress: state.uploadProgress.filter(
                    (fp: FileUploadProgress) => fp.id !== fileProgressId,
                  ),
                }))

                // Add placeholder file (this would be the actual uploaded file in real implementation)
                const mockFile: RAGInstanceFile = {
                  id: crypto.randomUUID(),
                  rag_instance_id: ragInstanceId,
                  file_id: crypto.randomUUID(),
                  processing_status: 'pending' as any,
                  rag_metadata: {},
                  created_at: new Date().toISOString(),
                  updated_at: new Date().toISOString(),
                }
                uploadedFiles.push(mockFile)
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

            // Update overall progress to complete and add files to RAG instance
            set(state => ({
              overallUploadProgress: 100,
              uploading: false,
              files: [...state.files, ...uploadedFiles],
            }))

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