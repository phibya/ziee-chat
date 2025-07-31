/**
 * RAG Provider API type definitions
 * Types for managing RAG providers in the application
 */

export interface RAGProviderProxySettings {
  enabled: boolean
  url: string
  username: string
  password: string
  no_proxy: string
  ignore_ssl_certificates: boolean
  proxy_ssl: boolean
  proxy_host_ssl: boolean
  peer_ssl: boolean
  host_ssl: boolean
}

export interface RAGProvider {
  id: string
  name: string
  type: RAGProviderType
  enabled: boolean
  api_key?: string
  base_url?: string
  proxy_settings?: RAGProviderProxySettings
  built_in?: boolean
  created_at?: string
  updated_at?: string
}

export type RAGProviderType =
  | 'local'
  | 'lightrag'
  | 'ragstack'
  | 'chroma'
  | 'weaviate'
  | 'pinecone'
  | 'custom'

export interface CreateRAGProviderRequest {
  name: string
  type: RAGProviderType
  enabled?: boolean
  api_key?: string
  base_url?: string
}

export interface UpdateRAGProviderRequest {
  name?: string
  enabled?: boolean
  api_key?: string
  base_url?: string
  proxy_settings?: RAGProviderProxySettings
}

export interface RAGProviderListResponse {
  providers: RAGProvider[]
  total: number
  page: number
  per_page: number
}

export interface RAGDatabaseCapabilities {
  semantic_search?: boolean
  hybrid_search?: boolean
  metadata_filtering?: boolean
  similarity_threshold?: boolean
}

export interface RAGDatabase {
  id: string
  provider_id: string
  name: string
  alias: string
  description?: string
  enabled: boolean
  is_active: boolean
  collection_name?: string
  embedding_model?: string
  chunk_size: number
  chunk_overlap: number
  capabilities?: RAGDatabaseCapabilities
  settings?: Record<string, any>
  created_at: string
  updated_at: string
}

export interface CreateRAGDatabaseRequest {
  name: string
  alias: string
  description?: string
  enabled?: boolean
  collection_name?: string
  embedding_model?: string
  chunk_size?: number
  chunk_overlap?: number
  capabilities?: RAGDatabaseCapabilities
  settings?: Record<string, any>
}

export interface UpdateRAGDatabaseRequest {
  name?: string
  alias?: string
  description?: string
  enabled?: boolean
  collection_name?: string
  embedding_model?: string
  chunk_size?: number
  chunk_overlap?: number
  capabilities?: RAGDatabaseCapabilities
  settings?: Record<string, any>
}

export interface RAGDatabaseListResponse {
  databases: RAGDatabase[]
  total: number
  page: number
  per_page: number
}