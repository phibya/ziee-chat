/**
 * Model API type definitions
 * Types for managing models in the application
 */

import type { SourceInfo } from './modelDownloads'

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

export interface MistralRsSettings {
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

export interface LlamaCppSettings {
  // Device configuration
  device_type?: string // Device type (cpu, cuda, metal, etc.)
  device_ids?: number[] // Array of device IDs for multi-GPU

  // Context & Memory Management (equivalent to MistralRs Sequence Management)
  ctx_size?: number // Context size (--ctx-size, default: 4096)
  batch_size?: number // Logical batch size (--batch-size, default: 2048)
  ubatch_size?: number // Physical batch size (--ubatch-size, default: 512)
  parallel?: number // Number of parallel sequences (--parallel, default: 1)
  keep?: number // Tokens to keep from initial prompt (--keep, default: 0)
  mlock?: boolean // Force model to stay in RAM (--mlock, default: false)
  no_mmap?: boolean // Disable memory mapping (--no-mmap, default: false)

  // Threading & Performance (equivalent to MistralRs Performance)
  threads?: number // Generation threads (--threads, default: -1)
  threads_batch?: number // Batch processing threads (--threads-batch, default: same as threads)
  cont_batching?: boolean // Enable continuous batching (--cont-batching, default: true)
  flash_attn?: boolean // Enable Flash Attention (--flash-attn, default: false)
  no_kv_offload?: boolean // Disable KV cache offloading (--no-kv-offload, default: false)

  // GPU Configuration (equivalent to MistralRs Device Config)
  n_gpu_layers?: number // Number of layers on GPU (--n-gpu-layers, default: 0)
  main_gpu?: number // Primary GPU index (--main-gpu, default: 0)
  split_mode?: 'none' | 'layer' | 'row' // How to split across GPUs (--split-mode)
  tensor_split?: string // GPU memory distribution ratios (--tensor-split)

  // Model Configuration (equivalent to MistralRs Model Config)
  rope_freq_base?: number // RoPE base frequency (--rope-freq-base)
  rope_freq_scale?: number // RoPE frequency scaling (--rope-freq-scale)
  rope_scaling?: 'none' | 'linear' | 'yarn' // RoPE scaling method (--rope-scaling)
  cache_type_k?: string // KV cache data type for K (--cache-type-k)
  cache_type_v?: string // KV cache data type for V (--cache-type-v)

  // Advanced Options
  seed?: number // Random seed (--seed, default: -1)
  numa?: 'distribute' | 'isolate' | 'numactl' // NUMA optimizations (--numa)
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
  engine_type: string // Engine type: "mistralrs" | "llamacpp" - REQUIRED
  engine_settings_mistralrs?: MistralRsSettings // MistralRs-specific settings
  engine_settings_llamacpp?: LlamaCppSettings // LlamaCpp-specific settings
  file_format: string // Model file format: "safetensors", "gguf", "bin", etc. - REQUIRED
  source?: SourceInfo // Source information for tracking download origin
  files?: ModelFileInfo[]
}

export interface CreateModelRequest {
  provider_id: string
  name: string
  alias: string
  description?: string
  enabled?: boolean
  capabilities?: ModelCapabilities
  parameters?: ModelParameters
  engine_type: string // Engine type: "mistralrs" | "llamacpp" - REQUIRED
  engine_settings_mistralrs?: MistralRsSettings // MistralRs-specific settings
  engine_settings_llamacpp?: LlamaCppSettings // LlamaCpp-specific settings
  file_format: string // Model file format: "safetensors", "gguf", "bin", etc. - REQUIRED
}

export interface UpdateModelRequest {
  name?: string
  alias?: string
  description?: string
  enabled?: boolean
  is_active?: boolean
  capabilities?: ModelCapabilities
  parameters?: ModelParameters
  engine_type?: string // Engine type: "mistralrs" | "llamacpp"
  engine_settings_mistralrs?: MistralRsSettings // MistralRs-specific settings
  engine_settings_llamacpp?: LlamaCppSettings // LlamaCpp-specific settings
  file_format?: string // Model file format
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
  engine_type: string // Engine type: "mistralrs" | "llamacpp" - REQUIRED
  engine_settings_mistralrs?: MistralRsSettings // MistralRs-specific settings
  engine_settings_llamacpp?: LlamaCppSettings // LlamaCpp-specific settings
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
