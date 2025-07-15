import { getBaseUrl, getAuthToken } from './core'

export interface UploadProgress {
  loaded: number
  total: number
  percentage: number
}

export interface UploadCallbacks {
  onProgress?: (progress: UploadProgress) => void
  onComplete?: (response: any) => void
  onError?: (error: string) => void
}

/**
 * Upload a single file with progress tracking
 */
export const uploadFile = async (
  modelId: string,
  file: File,
  mainFilename: string,
  callbacks?: UploadCallbacks,
): Promise<any> => {
  const baseUrl = await getBaseUrl()
  const token = getAuthToken()

  return new Promise((resolve, reject) => {
    const xhr = new XMLHttpRequest()
    const formData = new FormData()

    // Add file and metadata to form data
    formData.append('file', file)
    formData.append('filename', file.name)
    formData.append('main_filename', mainFilename)
    formData.append('file_size', file.size.toString())

    // Track upload progress
    xhr.upload.addEventListener('progress', event => {
      if (event.lengthComputable && callbacks?.onProgress) {
        const progress: UploadProgress = {
          loaded: event.loaded,
          total: event.total,
          percentage: Math.round((event.loaded / event.total) * 100),
        }
        callbacks.onProgress(progress)
      }
    })

    // Handle completion
    xhr.addEventListener('load', () => {
      if (xhr.status >= 200 && xhr.status < 300) {
        try {
          const response = JSON.parse(xhr.responseText)
          callbacks?.onComplete?.(response)
          resolve(response)
        } catch {
          const errorMsg = 'Failed to parse upload response'
          callbacks?.onError?.(errorMsg)
          reject(new Error(errorMsg))
        }
      } else {
        const errorMsg = `Upload failed with status ${xhr.status}: ${xhr.statusText}`
        callbacks?.onError?.(errorMsg)
        reject(new Error(errorMsg))
      }
    })

    // Handle errors
    xhr.addEventListener('error', () => {
      const errorMsg = 'Upload failed due to network error'
      callbacks?.onError?.(errorMsg)
      reject(new Error(errorMsg))
    })

    // Handle timeout
    xhr.addEventListener('timeout', () => {
      const errorMsg = 'Upload timed out'
      callbacks?.onError?.(errorMsg)
      reject(new Error(errorMsg))
    })

    // Configure request
    xhr.open(
      'POST',
      `${baseUrl}/api/admin/uploaded-models/${modelId}/upload-multipart`,
    )

    // Add auth header if available
    if (token) {
      xhr.setRequestHeader('Authorization', `Bearer ${token}`)
    }

    // Set timeout (10 minutes for large files)
    xhr.timeout = 10 * 60 * 1000

    // Start upload
    xhr.send(formData)
  })
}

/**
 * Upload multiple files sequentially with progress tracking
 */
export const uploadFiles = async (
  modelId: string,
  files: File[],
  mainFilename: string,
  callbacks?: {
    onFileProgress?: (fileIndex: number, progress: UploadProgress) => void
    onFileComplete?: (fileIndex: number, response: any) => void
    onFileError?: (fileIndex: number, error: string) => void
    onAllComplete?: () => void
    onOverallProgress?: (completedFiles: number, totalFiles: number) => void
  },
): Promise<any[]> => {
  const results: any[] = []

  for (let i = 0; i < files.length; i++) {
    const file = files[i]

    try {
      const result = await uploadFile(modelId, file, mainFilename, {
        onProgress: progress => callbacks?.onFileProgress?.(i, progress),
        onComplete: response => {
          callbacks?.onFileComplete?.(i, response)
          callbacks?.onOverallProgress?.(i + 1, files.length)
        },
        onError: error => callbacks?.onFileError?.(i, error),
      })

      results.push(result)
    } catch (error) {
      // Continue with remaining files even if one fails
      results.push({
        error: error instanceof Error ? error.message : 'Unknown error',
      })
    }
  }

  callbacks?.onAllComplete?.()
  return results
}

/**
 * Upload files with concurrent uploads (limited concurrency)
 */
export const uploadFilesConcurrent = async (
  modelId: string,
  files: File[],
  mainFilename: string,
  maxConcurrent: number = 3,
  callbacks?: {
    onFileProgress?: (fileIndex: number, progress: UploadProgress) => void
    onFileComplete?: (fileIndex: number, response: any) => void
    onFileError?: (fileIndex: number, error: string) => void
    onAllComplete?: () => void
    onOverallProgress?: (completedFiles: number, totalFiles: number) => void
  },
): Promise<any[]> => {
  const results: (any | null)[] = new Array(files.length).fill(null)
  let completedCount = 0

  // Create upload promise for a single file
  const uploadSingleFile = async (fileIndex: number): Promise<void> => {
    const file = files[fileIndex]

    try {
      const result = await uploadFile(modelId, file, mainFilename, {
        onProgress: progress =>
          callbacks?.onFileProgress?.(fileIndex, progress),
        onComplete: response => {
          callbacks?.onFileComplete?.(fileIndex, response)
          completedCount++
          callbacks?.onOverallProgress?.(completedCount, files.length)
        },
        onError: error => callbacks?.onFileError?.(fileIndex, error),
      })

      results[fileIndex] = result
    } catch (error) {
      results[fileIndex] = {
        error: error instanceof Error ? error.message : 'Unknown error',
      }
      completedCount++
      callbacks?.onOverallProgress?.(completedCount, files.length)
    }
  }

  // Execute uploads with limited concurrency
  const chunks: number[][] = []
  for (let i = 0; i < files.length; i += maxConcurrent) {
    chunks.push(files.slice(i, i + maxConcurrent).map((_, idx) => i + idx))
  }

  for (const chunk of chunks) {
    await Promise.all(chunk.map(uploadSingleFile))
  }

  callbacks?.onAllComplete?.()
  return results
}
