// Get file thumbnail
import { ApiClient } from '../api/client.ts'
import { useProjectsStore } from './projects.ts'
import { File } from '../types'

export const getFile = async (fileId: string): Promise<File> => {
  try {
    return await ApiClient.Files.get({ id: fileId })
  } catch (error) {
    console.error('Failed to fetch file:', error)
    throw error
  }
}

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

// Generate download token for a file
export const generateFileDownloadToken = async (
  fileId: string,
): Promise<{ token: string; expires_at: string }> => {
  try {
    const response = await ApiClient.Files.generateDownloadToken({ id: fileId })
    return response
  } catch (error) {
    console.error('Failed to generate download token:', error)
    throw error
  }
}

export const uploadFile = async (
  file: globalThis.File,
  progressCallback?: (progress: number) => void,
): Promise<File> => {
  const formData = new FormData()
  formData.append('file', file, file.name)

  const response = await ApiClient.Files.upload(formData, {
    fileUploadProgress: {
      onProgress: progressCallback,
    },
  })

  return response.file
}

export const deleteFile = async (fileId: string): Promise<void> => {
  try {
    await ApiClient.Files.delete({ id: fileId })

    // Remove from local state
    useProjectsStore.setState(state => ({
      files: state.files.filter(file => file.id !== fileId),
    }))
  } catch (error) {
    useProjectsStore.setState({
      filesError:
        error instanceof Error ? error.message : 'Failed to delete file',
    })
    throw error
  }
}
