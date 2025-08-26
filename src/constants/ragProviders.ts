import { RiDatabase2Line } from 'react-icons/ri'
import { FaServer, FaWrench } from 'react-icons/fa'
import { BsLightning } from 'react-icons/bs'
import type { IconType } from 'react-icons'
import { RAGProviderType, RAGEngineType } from '../types/api'

export interface RAGProviderOption {
  value: RAGProviderType
  label: string
}

export interface RAGProviderDefaults {
  base_url?: string
  settings?: Record<string, any>
}

export interface RAGEngineOption {
  value: RAGEngineType
  label: string
  description: string
}

export const SUPPORTED_RAG_PROVIDERS: RAGProviderOption[] = [
  { value: 'local', label: 'Local' },
  { value: 'lightrag', label: 'LightRAG' },
  { value: 'ragstack', label: 'RAGStack' },
  { value: 'chroma', label: 'Chroma' },
  { value: 'weaviate', label: 'Weaviate' },
  { value: 'pinecone', label: 'Pinecone' },
  { value: 'custom', label: 'Custom' },
]

export const RAG_PROVIDER_DEFAULTS: Record<
  RAGProviderType,
  RAGProviderDefaults
> = {
  local: {},
  lightrag: {},
  ragstack: {},
  chroma: {
    base_url: 'http://localhost:8000',
  },
  weaviate: {
    base_url: 'http://localhost:8080',
  },
  pinecone: {
    base_url: 'https://api.pinecone.io',
  },
  custom: {},
}

export const RAG_PROVIDER_ICONS: Record<RAGProviderType, IconType> = {
  local: FaServer,
  lightrag: BsLightning,
  ragstack: RiDatabase2Line,
  chroma: RiDatabase2Line,
  weaviate: RiDatabase2Line,
  pinecone: RiDatabase2Line,
  custom: FaWrench,
}

export const RAG_ENGINE_TYPES: RAGEngineOption[] = [
  {
    value: 'simple_vector',
    label: 'Simple Vector Search',
    description: 'Basic vector similarity search with embeddings',
  },
  {
    value: 'simple_graph',
    label: 'Simple Graph RAG',
    description: 'Graph-based RAG with entity and relationship extraction',
  },
]

// Default engine settings for different engine types
export const RAG_ENGINE_DEFAULTS = {
  simple_vector: {
    similarity_threshold: 0.7,
    max_results: 10,
    chunk_size: 1000,
    chunk_overlap: 200,
  },
  simple_graph: {
    similarity_threshold: 0.7,
    max_results: 10,
    community_level: 1,
    graph_depth: 2,
  },
}
