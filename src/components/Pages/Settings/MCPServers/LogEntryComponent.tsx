import type { MCPLogEntry } from '../../../../types/api'

interface LogEntryProps {
  entry: MCPLogEntry
}

export function LogEntryComponent({ entry }: LogEntryProps) {
  const formatTimestamp = (timestamp: string): string => {
    // Parse the UTC timestamp and convert to browser's local timezone
    const date = new Date(timestamp)

    return date.toLocaleString(undefined, {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
      hour12: false,
      timeZoneName: 'short'
    })
  }

  return `[${formatTimestamp(entry.timestamp)}] [${entry.log_type}] [${entry.level}] ${entry.message}`
}