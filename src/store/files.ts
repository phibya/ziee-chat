// Get file thumbnail
import { ApiClient } from '../api/client.ts'
// import { createProjectStore } from './project.ts' // For future use
import { File } from '../types'

export const getFile = async (fileId: string): Promise<File> => {
  try {
    return await ApiClient.Files.getFile({ file_id: fileId })
  } catch (error) {
    console.error('Failed to fetch file:', error)
    throw error
  }
}

export const getFileThumbnail = async (
  fileId: string,
): Promise<string | null> => {
  try {
    const response = await ApiClient.Files.getFilePreview({
      file_id: fileId,
      page: 1,
    })
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
      const response = await ApiClient.Files.getFilePreview({
        file_id: fileId,
        page,
      })
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
    const blob = await ApiClient.Files.downloadFile({ file_id: fileId })
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
    const response = await ApiClient.Files.generateDownloadToken({
      file_id: fileId,
    })
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

  const response = await ApiClient.Files.uploadFile(formData, {
    fileUploadProgress: {
      onProgress: progressCallback,
    },
  })

  return response.file
}

export const deleteFile = async (
  fileId: string,
  projectId?: string,
): Promise<void> => {
  try {
    await ApiClient.Files.deleteFile({ file_id: fileId })

    // Remove from local state if projectId provided
    if (projectId) {
      // Note: This needs to be implemented as a proper store action
      console.warn(
        'File deletion from project store needs proper action implementation',
      )
    }
  } catch (error) {
    if (projectId) {
      // Note: Error handling needs to be implemented as a proper store action
      console.warn(
        'File deletion error handling needs proper store action implementation',
      )
    }
    throw error
  }
}
