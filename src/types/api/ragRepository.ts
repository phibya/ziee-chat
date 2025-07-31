/**
 * RAG Repository API type definitions
 * Types for managing RAG repositories in the application
 */


export interface RAGRepository {
  id: string
  name: string
  description?: string
  url: string
  enabled: boolean
  requires_auth: boolean
  auth_token?: string
  priority: number
  created_at: string
  updated_at: string
}

export interface CreateRAGRepositoryRequest {
  name: string
  description?: string
  url: string
  enabled?: boolean
  requires_auth?: boolean
  auth_token?: string
  priority?: number
}

export interface UpdateRAGRepositoryRequest {
  name?: string
  description?: string
  url?: string
  enabled?: boolean
  requires_auth?: boolean
  auth_token?: string
  priority?: number
}

export interface RAGRepositoryListResponse {
  repositories: RAGRepository[]
  total: number
  page: number
  per_page: number
}

export interface RAGRepositoryConnectionTestResponse {
  success: boolean
  message: string
  available_databases_count?: number
}

export interface DownloadRAGDatabaseFromRepositoryRequest {
  repository_id: string
  database_id: string
  target_provider_id: string
  database_name?: string
  database_alias?: string
}