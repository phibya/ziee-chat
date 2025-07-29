// Document extraction types matching Rust backend

export interface DocumentModelParameters {
  context_size?: number
  gpu_layers?: number
  temperature?: number
  top_k?: number
  top_p?: number
  min_p?: number
  repeat_last_n?: number
  repeat_penalty?: number
  presence_penalty?: number
  frequency_penalty?: number
}

export interface SimpleExtractionSettings {
  // Empty object for simple extraction
}

export interface OcrExtractionSettings {
  language: string
  engine: string
}

export interface LlmExtractionSettings {
  model_id: string | null
  system_prompt: string
  parameters: DocumentModelParameters
}

export interface DocumentExtractionSettings {
  method: 'simple' | 'ocr' | 'llm'
  simple: SimpleExtractionSettings
  ocr: OcrExtractionSettings
  llm: LlmExtractionSettings
}

// API Request/Response types
export interface DocumentExtractionConfigResponse {
  settings: DocumentExtractionSettings
}

export interface SetMethodRequest {
  method: string
}

export interface SetOcrSettingsRequest {
  settings: OcrExtractionSettings
}

export interface SetLlmSettingsRequest {
  settings: LlmExtractionSettings
}

// File types that support extraction
export type ExtractionFileType = 'pdf' | 'image'

// Default settings
export const DEFAULT_OCR_SETTINGS: OcrExtractionSettings = {
  language: 'eng',
  engine: 'tesseract',
}

export const DEFAULT_MODEL_PARAMETERS: DocumentModelParameters = {
  context_size: 4096,
  gpu_layers: -1,
  temperature: 0.2,
  top_k: 20,
  top_p: 0.9,
  min_p: 0.1,
  repeat_last_n: 64,
  repeat_penalty: 1.05,
  presence_penalty: 0.1,
  frequency_penalty: 0.1,
}

export const DEFAULT_LLM_SETTINGS: LlmExtractionSettings = {
  model_id: null,
  system_prompt:
    'Extract all text from this document image. Maintain formatting and structure.',
  parameters: DEFAULT_MODEL_PARAMETERS,
}

export const DEFAULT_DOCUMENT_EXTRACTION_SETTINGS: DocumentExtractionSettings =
  {
    method: 'simple',
    simple: {},
    ocr: DEFAULT_OCR_SETTINGS,
    llm: DEFAULT_LLM_SETTINGS,
  }

// OCR language options
export const OCR_LANGUAGES = [
  { value: 'eng', label: 'English' },
  { value: 'fra', label: 'French' },
  { value: 'deu', label: 'German' },
  { value: 'spa', label: 'Spanish' },
  { value: 'ita', label: 'Italian' },
  { value: 'por', label: 'Portuguese' },
  { value: 'rus', label: 'Russian' },
  { value: 'jpn', label: 'Japanese' },
  { value: 'chi_sim', label: 'Chinese (Simplified)' },
  { value: 'chi_tra', label: 'Chinese (Traditional)' },
  { value: 'kor', label: 'Korean' },
  { value: 'ara', label: 'Arabic' },
  { value: 'hin', label: 'Hindi' },
] as const
