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
    // Initialize upload progress with unique IDs (append to existing)
    const newFileProgress = files.map(file => ({
      id: crypto.randomUUID(),
      filename: file.name,
      progress: 0,
      status: 'pending' as const,
      size: file.size,
    }))

    useProjectFilesStore.setState(state => ({
      uploading: true,
      uploadProgress: [...state.uploadProgress, ...newFileProgress],
      overallUploadProgress: 0,
      error: null,
      showProgress: true,
    }))

    const uploadedFiles: File[] = []

    // Upload files sequentially to better track progress
    for (let i = 0; i < files.length; i++) {
      const file = files[i]
      const fileProgressId = newFileProgress[i].id

      // Update current file status to uploading
      useProjectFilesStore.setState(state => ({
        uploadProgress: state.uploadProgress.map((fp: FileUploadProgress) =>
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
        const response = await ApiClient.Files.upload(formData, {
          fileUploadProgress: {
            onProgress: (progress: number) => {
              // Update file-specific progress
              useProjectFilesStore.setState(state => ({
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
              useProjectFilesStore.setState(state => ({
                uploadProgress: state.uploadProgress.filter(
                  (fp: FileUploadProgress) => fp.id !== fileProgressId,
                ),
              }))
            },
            onError: (error: string) => {
              // Mark this file as failed
              useProjectFilesStore.setState(state => ({
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
        })

        uploadedFiles.push(response.file)
      } catch (fileError) {
        // Mark this file as failed
        useProjectFilesStore.setState(state => ({
          uploadProgress: state.uploadProgress.map((fp: FileUploadProgress) =>
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

    debugger

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

    // Note: Completed files are automatically removed from progress on completion
    // Progress will be hidden when all uploads finish (no more progress items)

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
export const cancelProjectFileUpload = (): void => {
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

export const hideProjectUploadProgress = (): void => {
  useProjectFilesStore.setState({ showProgress: false })
}

export const showProjectUploadProgress = (): void => {
  useProjectFilesStore.setState({ showProgress: true })
}

// Get files for a specific project
export const getProjectFiles = (projectId: string): File[] => {
  const state = useProjectFilesStore.getState()
  return state.filesByProject[projectId] || []
}

// Remove a specific file upload progress by ID
export const removeProjectFileUploadProgress = (progressId: string): void => {
  useProjectFilesStore.setState(state => ({
    uploadProgress: state.uploadProgress.filter(
      (fp: FileUploadProgress) => fp.id !== progressId,
    ),
  }))
}

// Get a specific file upload progress by ID
export const getProjectFileUploadProgressById = (
  progressId: string,
): FileUploadProgress | undefined => {
  const state = useProjectFilesStore.getState()
  return state.uploadProgress.find(
    (fp: FileUploadProgress) => fp.id === progressId,
  )
}
