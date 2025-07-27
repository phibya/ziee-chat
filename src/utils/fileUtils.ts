/**
 * File utility functions
 */

/**
 * Formats file size in bytes to human readable format
 */
export const formatFileSize = (bytes: number): string => {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

/**
 * Determines if a file is a text file based on its extension
 */
export const isTextFile = (filename: string): boolean => {
  const textExtensions = [
    'txt',
    'md',
    'json',
    'js',
    'ts',
    'tsx',
    'jsx',
    'html',
    'css',
    'scss',
    'xml',
    'yml',
    'yaml',
    'csv',
    'log',
    'py',
    'java',
    'c',
    'cpp',
    'h',
    'hpp',
    'rs',
    'go',
    'php',
    'rb',
    'sh',
    'bat',
    'ps1',
    'sql',
    'r',
    'tex',
    'latex',
    'bib',
    'cfg',
    'conf',
    'ini',
    'toml',
  ]
  const extension = filename.split('.').pop()?.toLowerCase()
  return textExtensions.includes(extension || '')
}
