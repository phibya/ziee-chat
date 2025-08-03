import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../api/client'
import { Project } from '../types/api/projects'
import { File, FileUploadProgress } from '../types/api/files'

interface Conversation {
  id: string
  title: string
  project_id: string
  last_message: string
  message_count: number
  created_at: string
  updated_at: string
}

interface ProjectsState {
  // Data
  projects: Project[]
  currentProject: Project | null
  files: File[]
  conversations: Conversation[]

  // Loading states
  loading: boolean
  creating: boolean
  updating: boolean
  deleting: boolean
  uploading: boolean

  // File upload progress (current project only)
  uploadProgress: FileUploadProgress[]
  overallUploadProgress: number

  // File loading state (current project only)
  filesLoading: boolean
  filesError: string | null

  // Error state
  error: string | null
}

export const useProjectsStore = create<ProjectsState>()(
  subscribeWithSelector(
    (): ProjectsState => ({
      // Initial state
      projects: [],
      currentProject: null,
      files: [],
      conversations: [],
      loading: false,
      creating: false,
      updating: false,
      deleting: false,
      uploading: false,
      uploadProgress: [],
      overallUploadProgress: 0,
      filesLoading: false,
      filesError: null,
      error: null,
    }),
  ),
)

// Project actions
export const loadAllUserProjects = async (): Promise<void> => {
  try {
    useProjectsStore.setState({ loading: true, error: null })

    const response = await ApiClient.Projects.list({
      page: 1,
      per_page: 50,
    })

    useProjectsStore.setState({
      projects: response.projects,
      loading: false,
    })
  } catch (error) {
    useProjectsStore.setState({
      error: error instanceof Error ? error.message : 'Failed to load projects',
      loading: false,
    })
    throw error
  }
}

export const loadProjectById = async (id: string): Promise<void> => {
  try {
    useProjectsStore.setState({ loading: true, error: null })

    const response = await ApiClient.Projects.get({ project_id: id })

    useProjectsStore.setState({
      currentProject: response.project,
      loading: false,
      // Clear upload state when switching projects
      uploadProgress: [],
      uploading: false,
      overallUploadProgress: 0,
      filesError: null,
    })
  } catch (error) {
    useProjectsStore.setState({
      error: error instanceof Error ? error.message : 'Failed to load project',
      loading: false,
    })
    throw error
  }
}

export const loadProjectWithDetails = async (id: string): Promise<void> => {
  try {
    useProjectsStore.setState({ loading: true, error: null })

    const response = await ApiClient.Projects.get({ project_id: id })

    useProjectsStore.setState({
      currentProject: response.project,
      files: (response.project as any).files || [],
      conversations: (response.project as any).conversations || [],
      loading: false,
      // Clear upload state when switching projects
      uploadProgress: [],
      uploading: false,
      overallUploadProgress: 0,
      filesError: null,
    })
  } catch (error) {
    useProjectsStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to load project details',
      loading: false,
    })
    throw error
  }
}

export const createNewProject = async (data: {
  name: string
  description: string
}): Promise<Project> => {
  try {
    useProjectsStore.setState({ creating: true, error: null })

    const project = await ApiClient.Projects.create(data)

    useProjectsStore.setState(state => ({
      projects: [...state.projects, project],
      creating: false,
    }))

    return project
  } catch (error) {
    useProjectsStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to create project',
      creating: false,
    })
    throw error
  }
}

export const updateExistingProject = async (
  id: string,
  data: { name?: string; description?: string },
): Promise<Project> => {
  try {
    useProjectsStore.setState({ updating: true, error: null })

    const project = await ApiClient.Projects.update({
      project_id: id,
      ...data,
    })

    useProjectsStore.setState(state => ({
      projects: state.projects.map(p => (p.id === id ? project : p)),
      currentProject:
        state.currentProject?.id === id ? project : state.currentProject,
      updating: false,
    }))

    return project
  } catch (error) {
    useProjectsStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to update project',
      updating: false,
    })
    throw error
  }
}

export const deleteExistingProject = async (id: string): Promise<void> => {
  try {
    useProjectsStore.setState({ deleting: true, error: null })

    await ApiClient.Projects.delete({ project_id: id })

    useProjectsStore.setState(state => ({
      projects: state.projects.filter(p => p.id !== id),
      currentProject:
        state.currentProject?.id === id ? null : state.currentProject,
      deleting: false,
    }))
  } catch (error) {
    useProjectsStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to delete project',
      deleting: false,
    })
    throw error
  }
}

// File management functions

// Load files for a project
export const loadProjectFiles = async (projectId: string): Promise<void> => {
  try {
    useProjectsStore.setState({ filesLoading: true, filesError: null })

    const response = await ApiClient.Projects.listFiles({
      project_id: projectId,
      page: 1,
      per_page: 100,
    })

    useProjectsStore.setState({
      files: response.files,
      filesLoading: false,
    })
  } catch (error) {
    useProjectsStore.setState({
      filesError:
        error instanceof Error ? error.message : 'Failed to load project files',
      filesLoading: false,
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

    useProjectsStore.setState(state => ({
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
      useProjectsStore.setState(state => ({
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
        const response = await ApiClient.Projects.uploadFile(formData, {
          fileUploadProgress: {
            onProgress: (progress: number) => {
              // Update file-specific progress
              useProjectsStore.setState(state => ({
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
              useProjectsStore.setState(state => ({
                uploadProgress: state.uploadProgress.filter(
                  (fp: FileUploadProgress) => fp.id !== fileProgressId,
                ),
              }))
            },
            onError: (error: string) => {
              // Mark this file as failed
              useProjectsStore.setState(state => ({
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
        useProjectsStore.setState(state => ({
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

    // Update overall progress to complete and add files to project
    useProjectsStore.setState(state => ({
      overallUploadProgress: 100,
      uploading: false,
      files: [...state.files, ...uploadedFiles],
    }))

    // Note: Completed files are automatically removed from progress on completion

    return uploadedFiles
  } catch (error) {
    useProjectsStore.setState({
      filesError:
        error instanceof Error ? error.message : 'Failed to upload files',
      uploading: false,
      uploadProgress: [],
      overallUploadProgress: 0,
    })
    throw error
  }
}

// Utility actions
export const cancelProjectFileUpload = (): void => {
  useProjectsStore.setState({
    uploading: false,
    uploadProgress: [],
    overallUploadProgress: 0,
  })
}

export const clearFilesError = (): void => {
  useProjectsStore.setState({ filesError: null })
}

export const clearProjectsStoreError = (): void => {
  useProjectsStore.setState({ error: null })
}

// Helper functions

// Get files for a specific project
export const getProjectFiles = (projectId: string): File[] => {
  const state = useProjectsStore.getState()
  return state.files.filter(file => file.project_id === projectId)
}

// Remove a specific file upload progress by ID
export const removeProjectFileUploadProgress = (progressId: string): void => {
  useProjectsStore.setState(state => ({
    uploadProgress: state.uploadProgress.filter(
      (fp: FileUploadProgress) => fp.id !== progressId,
    ),
  }))
}

// Get a specific file upload progress by ID
export const getProjectFileUploadProgressById = (
  progressId: string,
): FileUploadProgress | undefined => {
  const state = useProjectsStore.getState()
  return state.uploadProgress.find(
    (fp: FileUploadProgress) => fp.id === progressId,
  )
}

export const resetProjectsStore = (): void => {
  useProjectsStore.setState({
    projects: [],
    currentProject: null,
    files: [],
    conversations: [],
    loading: false,
    creating: false,
    updating: false,
    deleting: false,
    uploading: false,
    uploadProgress: [],
    overallUploadProgress: 0,
    filesLoading: false,
    filesError: null,
    error: null,
  })
}
