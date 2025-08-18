export interface FileUploadProgress {
  id: string //random id
  filename: string
  progress: number
  status: 'pending' | 'uploading' | 'completed' | 'error'
  error?: string
  size?: number
}
