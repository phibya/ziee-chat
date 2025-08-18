import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../api/client'
import { File, Project } from '../types'
import { createStoreProxy } from '../utils/createStoreProxy.ts'
import { StoreApi, UseBoundStore } from 'zustand/index'
import { useEffect, useMemo, useRef } from 'react'
import { useParams } from 'react-router-dom'
import { debounce } from '../utils/debounce'
import { useProjectsStore } from './projects.ts'
import { FileUploadProgress } from '../types/client/file.ts'

export interface ProjectState {
  // Data
  project: Project | null
  files: File[]

  // Loading states
  loading: boolean
  updating: boolean
  uploading: boolean

  // File upload progress (current project only)
  uploadProgress: FileUploadProgress[]
  overallUploadProgress: number

  // File loading state (current project only)
  filesLoading: boolean
  filesError: string | null

  // Error state
  error: string | null

  // Store management
  destroy: () => void

  // Actions
  loadProject: () => Promise<void>
  updateProject: (data: {
    name?: string
    description?: string
    instruction?: string
  }) => Promise<Project>
  loadFiles: () => Promise<void>
  uploadFiles: (files: globalThis.File[]) => Promise<File[]>
  cancelFileUpload: () => void
  removeUploadProgress: (progressId: string) => void
  clearFilesError: () => void
  clearError: () => void
  reset: () => void
}

// Store map to keep the proxies
const ProjectStoreMap = new Map<
  string,
  ReturnType<typeof createStoreProxy<UseBoundStore<StoreApi<ProjectState>>>>
>()

// Map to track cleanup debounce functions for each project
const CleanupDebounceMap = new Map<string, ReturnType<typeof debounce>>()

export const createProjectStore = (project: string | Project) => {
  let projectId: string
  if (typeof project === 'string') {
    projectId = project
  } else {
    projectId = project.id
  }

  if (ProjectStoreMap.has(projectId)) {
    return ProjectStoreMap.get(projectId)!
  }

  const store = create<ProjectState>()(
    subscribeWithSelector(
      (set): ProjectState => ({
        // Initial state
        project: typeof project === 'string' ? null : project,
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
          ProjectStoreMap.delete(projectId)
        },

        // Actions
        loadProject: async () => {
          try {
            set({ loading: true, error: null })

            const response = await ApiClient.Projects.getProject({
              project_id: projectId,
            })

            set({
              project: response.project,
              loading: false,
              // Clear upload state when switching projects
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
                  : 'Failed to load project',
              loading: false,
            })
            throw error
          }
        },

        updateProject: async data => {
          try {
            set({ updating: true, error: null })

            const project = await ApiClient.Projects.updateProject({
              project_id: projectId,
              ...data,
            })

            set({
              project: project,
              updating: false,
            })

            useProjectsStore.setState({
              projects: useProjectsStore
                .getState()
                .projects.map(p => (p.id === projectId ? project : p)),
            })

            return project
          } catch (error) {
            set({
              error:
                error instanceof Error
                  ? error.message
                  : 'Failed to update project',
              updating: false,
            })
            throw error
          }
        },

        loadFiles: async () => {
          try {
            set({ filesLoading: true, filesError: null })

            const response = await ApiClient.Files.listProjectFiles({
              project_id: projectId,
              page: 1,
              per_page: 100,
            })

            set({
              files: response.files,
              filesLoading: false,
            })
          } catch (error) {
            set({
              filesError:
                error instanceof Error
                  ? error.message
                  : 'Failed to load project files',
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

            const uploadedFiles: File[] = []

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

              // Create FormData for the file
              const formData = new FormData()
              formData.append('file', file, file.name)
              formData.append('project_id', projectId)

              try {
                // Call the upload API with progress tracking using ApiClient.Files.upload
                const response = await ApiClient.Files.uploadProjectFile(
                  // @ts-ignore
                  formData,
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

                uploadedFiles.push(response.file)
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

            // Update overall progress to complete and add files to project
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
            project: null,
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
  ProjectStoreMap.set(projectId, storeProxy)

  // Immediately load project when store is created (except for default store)
  if (projectId !== 'default') {
    storeProxy.__state.loadProject().catch(error => {
      console.error(`Failed to auto-load project ${projectId}:`, error)
    })
    storeProxy.__state.loadFiles().catch(error => {
      console.error(
        `Failed to auto-load files for project ${projectId}:`,
        error,
      )
    })
  }

  return storeProxy
}

// Hook for components to use project-specific stores
export const useProjectStore = (projectId?: string) => {
  const { projectId: paramProjectId } = useParams<{
    projectId?: string
  }>()
  const currentId = projectId || paramProjectId
  const prevIdRef = useRef<string | undefined>(currentId)

  useEffect(() => {
    const prevId = prevIdRef.current

    // If projectId changed, set up debounced cleanup for the previous one
    if (prevId && prevId !== currentId) {
      // Create debounced cleanup function for the previous project
      const cleanupFn = debounce(
        () => {
          const store = ProjectStoreMap.get(prevId)
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
      // Return a default store for cases where there's no project ID
      return createProjectStore('default')
    }

    // Cancel any existing debounced cleanup for this project since it's being accessed again
    const existingCleanup = CleanupDebounceMap.get(id)
    if (existingCleanup) {
      existingCleanup.cancel()
      CleanupDebounceMap.delete(id)
    }

    return createProjectStore(id)
  }, [projectId, paramProjectId])
}
