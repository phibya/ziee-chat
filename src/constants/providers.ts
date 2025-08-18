import { ProviderType } from '../types'
import { RiAnthropicFill, RiOpenaiFill, RiGeminiFill } from 'react-icons/ri'
import { SiHuggingface } from 'react-icons/si'
import { FaServer, FaWrench } from 'react-icons/fa'
import { BsFillLightningChargeFill } from 'react-icons/bs'
import { DeepSeek, Mistral } from '@lobehub/icons'
import type { IconType } from 'react-icons'

export interface ProviderOption {
  value: ProviderType
  label: string
}

export interface ProviderDefaults {
  base_url?: string
  settings?: {
    // Provider-level infrastructure settings
    device?: string
    autoUnloadOldModels?: boolean
    parallelOperations?: number
    cpuThreads?: number
    huggingFaceAccessToken?: string
    // Legacy settings that may still be useful for advanced users
    contextShift?: boolean
    continuousBatching?: boolean
    threadsBatch?: number
    flashAttention?: boolean
    caching?: boolean
    kvCacheType?: string
    mmap?: boolean
  }
}

export const SUPPORTED_PROVIDERS: ProviderOption[] = [
  { value: 'local', label: 'Local' },
  { value: 'openai', label: 'OpenAI' },
  { value: 'anthropic', label: 'Anthropic' },
  { value: 'groq', label: 'Groq' },
  { value: 'gemini', label: 'Gemini' },
  { value: 'mistral', label: 'Mistral' },
  { value: 'deepseek', label: 'DeepSeek' },
  { value: 'huggingface', label: 'Hugging Face' },
  { value: 'custom', label: 'Custom' },
]

export const PROVIDER_DEFAULTS: Record<ProviderType, ProviderDefaults> = {
  openai: {
    base_url: 'https://api.openai.com/v1',
  },
  anthropic: {
    base_url: 'https://api.anthropic.com/v1',
  },
  groq: {
    base_url: 'https://api.groq.com/openai/v1',
  },
  gemini: {
    base_url: 'https://generativelanguage.googleapis.com/v1beta/openai',
  },
  mistral: {
    base_url: 'https://api.mistral.ai',
  },
  deepseek: {
    base_url: 'https://api.deepseek.com/v1',
  },
  huggingface: {
    base_url: 'https://api-inference.huggingface.co/v1',
  },
  local: {
    settings: {
      device: 'cpu',
      autoUnloadOldModels: true,
      parallelOperations: 1,
      cpuThreads: -1,
      huggingFaceAccessToken: '',
      contextShift: false,
      continuousBatching: false,
      threadsBatch: -1,
      flashAttention: true,
      caching: true,
      kvCacheType: 'q8_0',
      mmap: true,
    },
  },
  custom: {},
}

export const PROVIDER_ICONS: Record<ProviderType, IconType> = {
  local: FaServer,
  openai: RiOpenaiFill,
  anthropic: RiAnthropicFill,
  groq: BsFillLightningChargeFill,
  gemini: RiGeminiFill,
  mistral: Mistral,
  deepseek: DeepSeek,
  huggingface: SiHuggingface,
  custom: FaWrench,
}

export const KV_CACHE_TYPE_OPTIONS = [
  { value: 'q8_0', label: 'q8_0' },
  { value: 'q4_0', label: 'q4_0' },
  { value: 'q4_1', label: 'q4_1' },
  { value: 'q5_0', label: 'q5_0' },
  { value: 'q5_1', label: 'q5_1' },
]
