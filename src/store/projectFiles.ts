import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../api/client'
import type { File, FileUploadProgress } from '../types/api'

interface ProjectFilesState {
  // File data by project
  filesByProject: Record<string, File[]>

  // Upload progress
  uploading: boolean
  uploadProgress: FileUploadProgress[]
  overallUploadProgress: number

  // Loading states
  loading: boolean

  // Error state
  error: string | null

  // UI state
  showProgress: boolean
}

export const useProjectFilesStore = create<ProjectFilesState>()(
  subscribeWithSelector(
    (): ProjectFilesState => ({
      filesByProject: {},
      uploading: false,
      uploadProgress: [],
      overallUploadProgress: 0,
      loading: false,
      error: null,
      showProgress: false,
    }),
  ),
)

// Load files for a project
export const loadProjectFiles = async (projectId: string): Promise<void> => {
  try {
    useProjectFilesStore.setState({ loading: true, error: null })

    const response = await ApiClient.Projects.listFiles({
      project_id: projectId,
      page: 1,
      per_page: 100,
    })

    useProjectFilesStore.setState(state => ({
      filesByProject: {
        ...state.filesByProject,
        [projectId]: response.files,
      },
      loading: false,
    }))
  } catch (error) {
    useProjectFilesStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to load project files',
      loading: false,
    })
    throw error
  }
}

// Upload files to project with progress tracking
export const uploadFilesToProject = async (
  projectId: string,
  files: globalThis.File[],
): Promise<File[]> => {
  try {
    useProjectFilesStore.setState({
      uploading: true,
      uploadProgress: files.map(file => ({
        filename: file.name,
        progress: 0,
        status: 'pending' as const,
        size: file.size,
      })),
      overallUploadProgress: 0,
      error: null,
      showProgress: true,
    })

    const uploadedFiles: File[] = []

    // Upload files sequentially to better track progress
    for (let i = 0; i < files.length; i++) {
      const file = files[i]

      // Update current file status to uploading
      useProjectFilesStore.setState(state => ({
        uploadProgress: state.uploadProgress.map((fp, index) =>
          index === i ? { ...fp, status: 'uploading' as const } : fp,
        ),
      }))

      // Create FormData for the file
      const formData = new FormData()
      formData.append('file', file, file.name)
      formData.append('project_id', projectId)

      try {
        // Call the upload API with progress tracking
        const response = await ApiClient.Projects.uploadFile(formData, {
          fileUploadProgress: {
            onProgress: (progress: number) => {
              // Update file-specific progress
              useProjectFilesStore.setState(state => ({
                uploadProgress: state.uploadProgress.map((fp, index) =>
                  index === i ? { ...fp, progress: progress * 100 } : fp,
                ),
                overallUploadProgress:
                  (i * 100 + progress * 100) / files.length,
              }))
            },
            onComplete: () => {
              // Mark file as completed
              useProjectFilesStore.setState(state => ({
                uploadProgress: state.uploadProgress.map((fp, index) =>
                  index === i
                    ? { ...fp, progress: 100, status: 'completed' as const }
                    : fp,
                ),
              }))
            },
            onError: (error: string) => {
              // Mark this file as failed
              useProjectFilesStore.setState(state => ({
                uploadProgress: state.uploadProgress.map((fp, index) =>
                  index === i
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
        })

        uploadedFiles.push(response.file)
      } catch (fileError) {
        // Mark this file as failed
        useProjectFilesStore.setState(state => ({
          uploadProgress: state.uploadProgress.map((fp, index) =>
            index === i
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

    // Update overall progress to complete
    useProjectFilesStore.setState(state => ({
      overallUploadProgress: 100,
      uploading: false,
      // Update project files list
      filesByProject: {
        ...state.filesByProject,
        [projectId]: [
          ...(state.filesByProject[projectId] || []),
          ...uploadedFiles,
        ],
      },
    }))

    // Hide progress after a delay
    setTimeout(() => {
      useProjectFilesStore.setState({ showProgress: false })
    }, 2000)

    return uploadedFiles
  } catch (error) {
    useProjectFilesStore.setState({
      error: error instanceof Error ? error.message : 'Failed to upload files',
      uploading: false,
      uploadProgress: [],
      overallUploadProgress: 0,
      showProgress: false,
    })
    throw error
  }
}

// Delete a file
export const deleteProjectFile = async (
  projectId: string,
  fileId: string,
): Promise<void> => {
  try {
    await ApiClient.Files.delete({ id: fileId })

    // Remove from local state
    useProjectFilesStore.setState(state => ({
      filesByProject: {
        ...state.filesByProject,
        [projectId]: (state.filesByProject[projectId] || []).filter(
          file => file.id !== fileId,
        ),
      },
    }))
  } catch (error) {
    useProjectFilesStore.setState({
      error: error instanceof Error ? error.message : 'Failed to delete file',
    })
    throw error
  }
}

// Utility actions
export const cancelFileUpload = (): void => {
  useProjectFilesStore.setState({
    uploading: false,
    uploadProgress: [],
    overallUploadProgress: 0,
    showProgress: false,
  })
}

export const clearProjectFilesError = (): void => {
  useProjectFilesStore.setState({ error: null })
}

export const hideUploadProgress = (): void => {
  useProjectFilesStore.setState({ showProgress: false })
}

export const showUploadProgress = (): void => {
  useProjectFilesStore.setState({ showProgress: true })
}

// Get files for a specific project
export const getProjectFiles = (projectId: string): File[] => {
  const state = useProjectFilesStore.getState()
  return state.filesByProject[projectId] || []
}

// Get file thumbnail
export const getFileThumbnail = async (
  fileId: string,
): Promise<string | null> => {
  try {
    const response = await ApiClient.Files.preview({ id: fileId, page: 1 })
    return window.URL.createObjectURL(response)
  } catch (_error) {
    console.debug('Thumbnail not available for file:', fileId)
    return null
  }
}

// Get multiple file thumbnails (up to 5)
export const getFileThumbnails = async (
  fileId: string,
  thumbnailCount: number,
): Promise<string[]> => {
  const maxThumbnails = Math.min(thumbnailCount, 5)
  const thumbnails: string[] = []

  for (let page = 1; page <= maxThumbnails; page++) {
    try {
      const response = await ApiClient.Files.preview({ id: fileId, page })
      const url = window.URL.createObjectURL(response)
      thumbnails.push(url)
    } catch (_error) {
      console.debug(`Thumbnail ${page} not available for file:`, fileId)
      break // Stop if a thumbnail is not available
    }
  }

  return thumbnails
}

// Get file content for text files
export const getFileContent = async (fileId: string): Promise<string> => {
  try {
    const blob = await ApiClient.Files.download({ id: fileId })
    // Convert blob to text
    return await blob.text()
  } catch (error) {
    console.error('Failed to fetch file content:', error)
    throw error
  }
}
