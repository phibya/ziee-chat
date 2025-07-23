/**
 * Common types used across the application
 */

/**
 * Supported application languages
 */
export type SupportedLanguage = 'en' | 'vi'

/**
 * Type guard to check if a string is a supported language
 */
export function isSupportedLanguage(value: string): value is SupportedLanguage {
  return value === 'en' || value === 'vi'
}

/**
 * Language display names for UI
 */
export const LANGUAGE_NAMES: Record<SupportedLanguage, string> = {
  en: 'English',
  vi: 'Tiếng Việt',
}

/**
 * Available language options for forms/selectors
 */
export const LANGUAGE_OPTIONS = [
  { value: 'en' as const, label: LANGUAGE_NAMES.en },
  { value: 'vi' as const, label: LANGUAGE_NAMES.vi },
]
