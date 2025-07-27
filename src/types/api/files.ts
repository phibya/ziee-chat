export interface File {
  id: string
  user_id: string
  filename: string
  file_path: string
  file_size: number
  mime_type?: string
  checksum?: string
  project_id?: string
  thumbnail_count: number
  processing_metadata: Record<string, any>
  created_at: string
  updated_at: string
}

export interface UploadFileResponse {
  file: File
}

export interface FileListResponse {
  files: File[]
  total: number
  page: number
  per_page: number
}

export interface FileListParams {
  page?: number
  per_page?: number
  search?: string
}

export interface FileUploadProgress {
  filename: string
  progress: number
  status: 'pending' | 'uploading' | 'completed' | 'error'
  error?: string
  size?: number
}

export interface DownloadTokenResponse {
  token: string
  expires_at: string
}
