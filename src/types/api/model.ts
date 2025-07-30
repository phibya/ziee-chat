/**
 * Model API type definitions
 * Types for managing models in the application
 */

export interface ModelCapabilities {
  vision?: boolean
  audio?: boolean
  tools?: boolean
  code_interpreter?: boolean
}

export interface ModelParameters {
  // Context and generation parameters
  max_tokens?: number

  // Sampling parameters
  temperature?: number
  top_k?: number
  top_p?: number
  min_p?: number

  // Repetition control
  repeat_last_n?: number
  repeat_penalty?: number
  presence_penalty?: number
  frequency_penalty?: number

  // Generation control
  seed?: number // Random seed for reproducible outputs
  stop?: string[] // Stop sequences to terminate generation
}

export interface ModelSettings {
  // Device configuration
  device_type?: string // Device type (cpu, cuda, metal, etc.)
  device_ids?: number[] // Array of device IDs for multi-GPU

  // Sequence and memory management
  max_seqs?: number // Maximum running sequences at any time
  max_seq_len?: number // Maximum sequence length
  no_kv_cache?: boolean // Use no KV cache
  truncate_sequence?: boolean // Truncate sequences that exceed max length

  // PagedAttention configuration
  paged_attn_gpu_mem?: number // GPU memory for KV cache in MBs
  paged_attn_gpu_mem_usage?: number // GPU memory usage percentage 0-1
  paged_ctxt_len?: number // Total context length for KV cache
  paged_attn_block_size?: number // PagedAttention block size
  no_paged_attn?: boolean // Disable PagedAttention on CUDA
  paged_attn?: boolean // Enable PagedAttention on Metal

  // Performance optimization
  prefix_cache_n?: number // Number of prefix caches to hold
  prompt_chunksize?: number // Prompt batching chunk size

  // Model configuration
  dtype?: string // Model data type: auto, f16, f32, bf16
  in_situ_quant?: string // In-situ quantization method

  // Reproducibility
  seed?: number // Seed for reproducible generation

  // Vision model parameters
  max_edge?: number // Maximum edge length for image resizing
  max_num_images?: number // Maximum number of images
  max_image_length?: number // Maximum image edge length
}

export interface ModelFileInfo {
  filename: string
  file_size_bytes: number
  file_type: string
  uploaded_at: string
}

export interface Model {
  id: string
  provider_id: string
  name: string
  alias: string
  description?: string
  enabled: boolean
  is_deprecated: boolean
  is_active: boolean
  capabilities?: ModelCapabilities
  parameters?: ModelParameters
  created_at: string
  updated_at: string
  // Additional fields for Candle models (undefined for other providers)
  file_size_bytes?: number
  validation_status?: string
  validation_issues?: string[]
  port?: number // Port number where the model server is running
  pid?: number // Process ID of the running model server
  settings?: ModelSettings // Model-specific performance settings
  files?: ModelFileInfo[]
}

export interface CreateModelRequest {
  provider_id: string
  name: string
  alias: string
  description?: string
  enabled?: boolean
  capabilities?: ModelCapabilities
  settings?: ModelSettings
}

export interface UpdateModelRequest {
  name?: string
  alias?: string
  description?: string
  enabled?: boolean
  is_active?: boolean
  capabilities?: ModelCapabilities
  parameters?: ModelParameters
  settings?: ModelSettings
}

export interface ModelFile {
  id: string
  model_id: string
  filename: string
  file_path: string
  file_size_bytes: number
  file_type: string
  upload_status: string
  uploaded_at: string
}

export interface ModelUploadResponse {
  model_id: string
  upload_url?: string
  chunk_uploaded: boolean
  upload_complete: boolean
  next_chunk_index?: number
}

export interface ModelListResponse {
  models: Model[]
  total: number
  page: number
  per_page: number
  total_storage_bytes: number
}

export interface ModelDetailsResponse {
  model: Model
  files: ModelFileInfo[]
  storage_size_bytes: number
  validation_issues: string[]
}

export interface ModelValidationResult {
  is_valid: boolean
  issues: string[]
  required_files: string[]
  present_files: string[]
}

export interface ModelStatusCounts {
  active: number
  inactive: number
  deprecated: number
  enabled: number
  disabled: number
}

export interface ModelStorageInfo {
  provider_id: string
  total_models: number
  total_storage_bytes: number
  models_by_status: ModelStatusCounts
}

// Request types for model operations
export interface AddModelToProviderRequest {
  name: string
  alias: string
  description?: string
  path?: string
  enabled?: boolean
  capabilities?: ModelCapabilities
  parameters?: ModelParameters
  settings?: ModelSettings
}

export interface RemoveModelFromProviderRequest {
  providerId: string
  modelId: string
}

// Model runtime operations
export interface StartModelRequest {
  modelId: string
}

export interface StopModelRequest {
  modelId: string
}

export interface ModelRuntimeInfo {
  pid?: number
  port?: number
  is_active: boolean
}

// Upload related types
export interface ModelUploadChunk {
  chunk_index: number
  chunk_data: ArrayBuffer
  is_final_chunk: boolean
}

export interface ModelUploadStatus {
  model_id: string
  total_chunks: number
  uploaded_chunks: number
  upload_complete: boolean
  validation_status?: string
  validation_issues?: string[]
}
