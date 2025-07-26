/**
 * Utility functions for formatting download-related data
 */

/**
 * Format bytes into human-readable format
 * @param bytes - Number of bytes to format
 * @returns Formatted string with appropriate unit (B, KB, MB, GB)
 */
export const formatBytes = (bytes: number): string => {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`
}

/**
 * Format download speed in bytes per second to human-readable format
 * @param speedBps - Speed in bytes per second
 * @returns Formatted string with appropriate unit (KB/s, MB/s, GB/s)
 */
export const formatSpeed = (speedBps?: number): string => {
  if (!speedBps || speedBps === 0) return ''

  const k = 1024
  if (speedBps >= k * k * k) {
    // GB/s
    return `${(speedBps / (k * k * k)).toFixed(1)} GB/s`
  } else if (speedBps >= k * k) {
    // MB/s
    return `${(speedBps / (k * k)).toFixed(1)} MB/s`
  } else if (speedBps >= k) {
    // KB/s
    return `${(speedBps / k).toFixed(1)} KB/s`
  } else {
    // B/s
    return `${speedBps.toFixed(0)} B/s`
  }
}

/**
 * Format ETA (estimated time of arrival) in seconds to human-readable format
 * @param etaSeconds - ETA in seconds
 * @returns Formatted string (e.g., "2h 30m", "5m 20s", "45s")
 */
export const formatETA = (etaSeconds?: number): string => {
  if (!etaSeconds || etaSeconds <= 0) return ''

  const hours = Math.floor(etaSeconds / 3600)
  const minutes = Math.floor((etaSeconds % 3600) / 60)
  const seconds = Math.floor(etaSeconds % 60)

  if (hours > 0) {
    return `${hours}h ${minutes}m`
  } else if (minutes > 0) {
    return `${minutes}m ${seconds}s`
  } else {
    return `${seconds}s`
  }
}

/**
 * Calculate download progress percentage
 * @param current - Current bytes downloaded
 * @param total - Total bytes to download
 * @returns Progress percentage (0-100)
 */
export const calculateProgress = (current: number, total: number): number => {
  if (total === 0) return 0
  return Math.min(100, Math.max(0, (current / total) * 100))
}

/**
 * Format download progress as a readable string
 * @param current - Current bytes downloaded
 * @param total - Total bytes to download
 * @returns Formatted progress string (e.g., "150.5 MB / 2.1 GB (7.2%)")
 */
export const formatProgress = (current: number, total: number): string => {
  const percentage = calculateProgress(current, total)
  return `${formatBytes(current)} / ${formatBytes(total)} (${percentage.toFixed(1)}%)`
}

/**
 * Estimate remaining time based on current progress and speed
 * @param current - Current bytes downloaded
 * @param total - Total bytes to download
 * @param speedBps - Download speed in bytes per second
 * @returns Estimated remaining time in seconds, or null if cannot be calculated
 */
export const estimateRemainingTime = (
  current: number,
  total: number,
  speedBps: number,
): number | null => {
  if (speedBps <= 0 || current >= total) return null

  const remaining = total - current
  return Math.round(remaining / speedBps)
}
