/**
 * Supported model architectures and file types for Local provider
 */

export interface ModelArchitecture {
  key: string
  label: string
  description: string
  supportedFormats?: string[]
}

export interface ModelFileType {
  key: string
  label: string
  description: string
  extensions: string[]
  mimeTypes?: string[]
}

export const LOCAL_MODEL_ARCHITECTURES: ModelArchitecture[] = [
  {
    key: 'llama',
    label: 'LLaMA',
    description: 'Meta LLaMA models (LLaMA 1, LLaMA 2, Code Llama)',
    supportedFormats: ['GGUF', 'SafeTensors', 'PyTorch'],
  },
  {
    key: 'mistral',
    label: 'Mistral',
    description: 'Mistral AI models (Mistral 7B, Mixtral)',
    supportedFormats: ['GGUF', 'SafeTensors'],
  },
  {
    key: 'gemma',
    label: 'Gemma',
    description: 'Google Gemma models',
    supportedFormats: ['GGUF', 'SafeTensors'],
  },
  {
    key: 'phi',
    label: 'Phi',
    description: 'Microsoft Phi models (Phi-2, Phi-3)',
    supportedFormats: ['GGUF', 'SafeTensors'],
  },
  {
    key: 'qwen',
    label: 'Qwen',
    description: 'Alibaba Qwen models',
    supportedFormats: ['GGUF', 'SafeTensors'],
  },
  {
    key: 'stable-lm',
    label: 'StableLM',
    description: 'Stability AI StableLM models',
    supportedFormats: ['GGUF', 'SafeTensors'],
  },
]

// Convert to options format for Select component
export const LOCAL_ARCHITECTURE_OPTIONS = LOCAL_MODEL_ARCHITECTURES.map(
  arch => ({
    value: arch.key,
    label: arch.label,
    description: arch.description,
  }),
)

// Supported file types for Local models
export const LOCAL_FILE_TYPES: ModelFileType[] = [
  {
    key: 'safetensors',
    label: 'SafeTensors (.safetensors)',
    description:
      'Safe tensor format with metadata validation and memory mapping support',
    extensions: ['.safetensors'],
    mimeTypes: ['application/octet-stream'],
  },
  {
    key: 'pytorch',
    label: 'PyTorch Binary (.bin)',
    description: 'Traditional PyTorch binary format',
    extensions: ['.bin', '.pt', '.pth'],
    mimeTypes: ['application/octet-stream'],
  },
  {
    key: 'gguf',
    label: 'GGUF (.gguf)',
    description: 'GGML Universal Format for quantized models',
    extensions: ['.gguf'],
    mimeTypes: ['application/octet-stream'],
  },
]

// Convert to options format for Select component
export const LOCAL_FILE_TYPE_OPTIONS = LOCAL_FILE_TYPES.map(type => ({
  value: type.key,
  label: type.label,
  description: type.description,
  extensions: type.extensions,
}))

// Default architecture and file type
export const DEFAULT_LOCAL_ARCHITECTURE = 'llama'
export const DEFAULT_LOCAL_FILE_TYPE = 'safetensors'
