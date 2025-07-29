import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../api/client'
import type {
  DocumentExtractionSettings,
  ExtractionFileType,
  OcrExtractionSettings,
  LlmExtractionSettings,
} from '../types/api/document-extraction'
import { DEFAULT_DOCUMENT_EXTRACTION_SETTINGS } from '../types/api/document-extraction'

interface DocumentExtractionState {
  // Settings by file type
  pdfSettings: DocumentExtractionSettings
  imageSettings: DocumentExtractionSettings

  // Loading states
  loading: boolean
  error: string | null

  // Initialization state
  initialized: boolean
}

export const useDocumentExtractionStore = create<DocumentExtractionState>()(
  subscribeWithSelector((_set, _get) => ({
    pdfSettings: DEFAULT_DOCUMENT_EXTRACTION_SETTINGS,
    imageSettings: {
      ...DEFAULT_DOCUMENT_EXTRACTION_SETTINGS,
      method: 'ocr', // Default for images since they don't support 'simple'
    },
    loading: false,
    error: null,
    initialized: false,
  })),
)

// Store methods - defined OUTSIDE the store definition
export const initializeDocumentExtraction = async () => {
  useDocumentExtractionStore.setState({ loading: true, error: null })

  try {
    // Load settings for both PDF and image
    const [pdfResponse, imageResponse] = await Promise.all([
      ApiClient.Admin.getExtractionConfig({ file_type: 'pdf' }),
      ApiClient.Admin.getExtractionConfig({ file_type: 'image' }),
    ])

    useDocumentExtractionStore.setState({
      pdfSettings: pdfResponse.settings,
      imageSettings: imageResponse.settings,
      initialized: true,
      loading: false,
      error: null,
    })
  } catch (error) {
    console.error('Document extraction initialization failed:', error)
    useDocumentExtractionStore.setState({
      loading: false,
      error: error instanceof Error ? error.message : 'Unknown error',
      initialized: false,
    })
    throw error
  }
}

export const setExtractionMethod = async (
  fileType: ExtractionFileType,
  method: 'simple' | 'ocr' | 'llm',
): Promise<void> => {
  try {
    const response = await ApiClient.Admin.setExtractionMethod({
      file_type: fileType,
      method,
    })

    // Update local state
    const stateKey = fileType === 'pdf' ? 'pdfSettings' : 'imageSettings'
    useDocumentExtractionStore.setState({
      [stateKey]: response.settings,
    })
  } catch (error) {
    console.error(`Failed to set extraction method for ${fileType}:`, error)
    throw error
  }
}

export const setOcrSettings = async (
  fileType: ExtractionFileType,
  settings: OcrExtractionSettings,
): Promise<void> => {
  try {
    const response = await ApiClient.Admin.setOcrSettings({
      file_type: fileType,
      settings,
    })

    // Update local state
    const stateKey = fileType === 'pdf' ? 'pdfSettings' : 'imageSettings'
    useDocumentExtractionStore.setState({
      [stateKey]: response.settings,
    })
  } catch (error) {
    console.error(`Failed to set OCR settings for ${fileType}:`, error)
    throw error
  }
}

export const setLlmSettings = async (
  fileType: ExtractionFileType,
  settings: LlmExtractionSettings,
): Promise<void> => {
  try {
    const response = await ApiClient.Admin.setLlmSettings({
      file_type: fileType,
      settings,
    })

    // Update local state
    const stateKey = fileType === 'pdf' ? 'pdfSettings' : 'imageSettings'
    useDocumentExtractionStore.setState({
      [stateKey]: response.settings,
    })
  } catch (error) {
    console.error(`Failed to set LLM settings for ${fileType}:`, error)
    throw error
  }
}

// Helper functions for getting settings
export const getSettingsForFileType = (
  fileType: ExtractionFileType,
): DocumentExtractionSettings => {
  const state = useDocumentExtractionStore.getState()
  return fileType === 'pdf' ? state.pdfSettings : state.imageSettings
}

export const getCurrentMethod = (fileType: ExtractionFileType): string => {
  const settings = getSettingsForFileType(fileType)
  return settings.method
}

export const getOcrSettingsForFileType = (
  fileType: ExtractionFileType,
): OcrExtractionSettings => {
  const settings = getSettingsForFileType(fileType)
  return settings.ocr
}

export const getLlmSettingsForFileType = (
  fileType: ExtractionFileType,
): LlmExtractionSettings => {
  const settings = getSettingsForFileType(fileType)
  return settings.llm
}

// Validation helpers
export const validateLlmSettings = (
  settings: LlmExtractionSettings,
): string[] => {
  const errors: string[] = []

  if (!settings.model_id) {
    errors.push('Model ID is required')
  }

  if (!settings.system_prompt.trim()) {
    errors.push('System prompt is required')
  }

  if (settings.parameters.temperature !== undefined) {
    if (
      settings.parameters.temperature < 0 ||
      settings.parameters.temperature > 2
    ) {
      errors.push('Temperature must be between 0 and 2')
    }
  }

  if (settings.parameters.top_p !== undefined) {
    if (settings.parameters.top_p < 0 || settings.parameters.top_p > 1) {
      errors.push('Top-p must be between 0 and 1')
    }
  }

  return errors
}

export const validateOcrSettings = (
  settings: OcrExtractionSettings,
): string[] => {
  const errors: string[] = []

  if (!settings.language) {
    errors.push('OCR language is required')
  }

  if (!settings.engine) {
    errors.push('OCR engine is required')
  }

  return errors
}
