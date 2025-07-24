/**
 * Download instance types for model downloads from repositories
 */

import { ModelCapabilities, ModelSettings } from './model'

export interface DownloadProgress {
  phase: string
  current: number
  total: number
  message: string
  current_bytes?: number
  total_bytes?: number
  current_file?: string
  total_files?: number
  files_completed?: number
  download_speed?: number
  eta_seconds?: number
}

export interface DownloadFromRepositoryRequest {
  provider_id: string
  repository_id: string
  repository_path: string
  main_filename: string
  repository_branch?: string
  name: string
  alias: string
  description?: string
  file_format: string
  capabilities?: ModelCapabilities
  settings?: ModelSettings
}

export interface DownloadInstance {
  id: string
  provider_id: string
  repository_id: string
  request_data: DownloadFromRepositoryRequest
  status: 'pending' | 'downloading' | 'completed' | 'failed' | 'cancelled'
  progress_data: DownloadProgress | null
  error_message: string | null
  started_at: string
  completed_at?: string
  model_id?: string
  created_at: string
  updated_at: string
}

export interface DownloadProgressUpdate {
  id: string
  status: string
  phase?: string | null
  current?: number | null
  total?: number | null
  message?: string | null
  speed_bps?: number | null
  eta_seconds?: number | null
  error_message?: string | null
}

export interface DownloadInstanceListResponse {
  downloads: DownloadInstance[]
  total: number
  page: number
  per_page: number
}

export interface CancelDownloadRequest {
  id: string
}

export interface DownloadStatusSummary {
  active: number
  completed: number
  failed: number
  cancelled: number
  total: number
}

export interface CreateDownloadInstanceRequest {
  provider_id: string
  repository_id: string
  request_data: DownloadFromRepositoryRequest
}

export interface UpdateDownloadProgressRequest {
  progress_data: DownloadProgress
}

export interface UpdateDownloadStatusRequest {
  status: 'pending' | 'downloading' | 'completed' | 'failed' | 'cancelled'
  error_message?: string
  model_id?: string
}