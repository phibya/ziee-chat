/**
 * Repository API type definitions
 * Types for managing model repositories in the application
 */

export interface RepositoryAuthConfig {
  api_key?: string
  username?: string
  password?: string
  token?: string
}

export interface Repository {
  id: string
  name: string
  url: string
  auth_type: 'none' | 'api_key' | 'basic_auth' | 'bearer_token'
  auth_config?: RepositoryAuthConfig
  enabled: boolean
  built_in?: boolean
  created_at?: string
  updated_at?: string
}

export interface CreateRepositoryRequest {
  name: string
  url: string
  auth_type: 'none' | 'api_key' | 'basic_auth' | 'bearer_token'
  auth_config?: RepositoryAuthConfig
  enabled?: boolean
}

export interface UpdateRepositoryRequest {
  name?: string
  url?: string
  auth_type?: 'none' | 'api_key' | 'basic_auth' | 'bearer_token'
  auth_config?: RepositoryAuthConfig
  enabled?: boolean
}

export interface RepositoryListResponse {
  repositories: Repository[]
  total: number
  page: number
  per_page: number
}

export interface TestRepositoryConnectionRequest {
  name: string
  url: string
  auth_type: 'none' | 'api_key' | 'basic_auth' | 'bearer_token'
  auth_config?: RepositoryAuthConfig
}

export interface TestRepositoryConnectionResponse {
  success: boolean
  message: string
}