import { useEffect, useRef, useMemo } from 'react'
import { Button, Select, Flex, Tooltip, Alert, Typography, theme } from 'antd'
import { DivScrollY } from '../../../common/DivScrollY'
import {
  FileTextOutlined,
  VerticalAlignBottomOutlined,
  ClearOutlined,
  ReloadOutlined,
} from '@ant-design/icons'
import type { MCPServer, MCPLogType } from '../../../../types/api'
import { useMCPLogsStore } from '../../../../store/mcpLogs'
import { LogEntryComponent } from './LogEntryComponent'

interface MCPLogsPanelProps {
  server: MCPServer
  isEditable?: boolean
}

export function MCPLogsPanel({ server }: MCPLogsPanelProps) {
  const logsEndRef = useRef<HTMLDivElement>(null)
  const { token } = theme.useToken()

  // Use server-specific store with direct destructuring
  const {
    logs,
    connection,
    preferences,
    subscribeToLogs,
    disconnectFromLogs,
    clearLogs,
    updatePreferences,
  } = useMCPLogsStore(server.id)

  // Filter logs based on selected types
  const filteredLogs = useMemo(() => {
    if (
      !preferences?.selectedLogTypes ||
      preferences.selectedLogTypes.length === 0
    ) {
      return logs
    }
    return logs.filter(log =>
      preferences.selectedLogTypes.includes(log.log_type),
    )
  }, [logs, preferences.selectedLogTypes])

  // Connect to logs when panel opens
  useEffect(() => {
    if (!connection.connected && !connection.loading) {
      subscribeToLogs()
    }

    return () => {
      // Cleanup when panel closes
      disconnectFromLogs()
    }
  }, [server.id, subscribeToLogs, disconnectFromLogs])

  // Auto-scroll to bottom when new logs arrive
  useEffect(() => {
    if (preferences.autoScroll && logsEndRef.current) {
      logsEndRef.current.scrollIntoView({ behavior: 'smooth' })
    }
  }, [filteredLogs, preferences.autoScroll])

  const handleToggleAutoScroll = () => {
    updatePreferences({ autoScroll: !preferences.autoScroll })
  }

  const handleClearLogs = () => {
    clearLogs()
  }

  const handleReconnect = () => {
    disconnectFromLogs()
    setTimeout(() => {
      subscribeToLogs()
    }, 100)
  }

  const handleLogTypeChange = (selectedTypes: MCPLogType[]) => {
    updatePreferences({ selectedLogTypes: selectedTypes })
  }

  const logTypeOptions = [
    { label: 'Exec', value: 'Exec' },
    { label: 'In', value: 'In' },
    { label: 'Out', value: 'Out' },
    { label: 'Err', value: 'Err' },
  ]

  return (
    <div className="mt-4">
      {/* Header */}
      <Flex justify="space-between" align="center" className="mb-3">
        <span className="flex items-center gap-2">
          <FileTextOutlined />
          Server Logs
        </span>

        <Flex gap={8} align="center">
          <Tooltip title="Auto-scroll to latest logs">
            <Button
              type={preferences.autoScroll ? 'primary' : 'default'}
              size="small"
              icon={<VerticalAlignBottomOutlined />}
              onClick={handleToggleAutoScroll}
            >
              Auto Scroll
            </Button>
          </Tooltip>

          <Button
            type="text"
            size="small"
            icon={<ClearOutlined />}
            onClick={handleClearLogs}
            disabled={logs.length === 0}
          >
            Clear
          </Button>

          <Button
            type="text"
            size="small"
            icon={<ReloadOutlined />}
            onClick={handleReconnect}
            loading={connection.loading}
          >
            Reconnect
          </Button>
        </Flex>
      </Flex>

      {/* Error Display */}
      {connection.error && (
        <Alert
          message={connection.error}
          type="error"
          className="mb-3"
          showIcon
        />
      )}

      {/* Log Type Filter */}
      <Flex wrap="wrap" gap={8} align="center" className="mb-3">
        <Typography.Text>Filter by log type:</Typography.Text>
        <Select
          mode="multiple"
          placeholder="Select log types"
          value={preferences.selectedLogTypes}
          onChange={handleLogTypeChange}
          options={logTypeOptions}
          size="small"
          style={{ minWidth: 200 }}
        />
      </Flex>

      {/* Logs Display */}
      <DivScrollY
        style={{
          maxHeight: '400px',
          backgroundColor: token.colorBgLayout,
        }}
        className="w-full mt-2 p-1 rounded"
      >
        <pre className="font-mono text-xs whitespace-pre-wrap">
          {connection.loading && filteredLogs.length === 0
            ? 'Loading logs...'
            : filteredLogs.length === 0
              ? 'No logs to display'
              : filteredLogs
                  .map(log => LogEntryComponent({ entry: log }))
                  .join('\n')}
          <div ref={logsEndRef} />
        </pre>
      </DivScrollY>
    </div>
  )
}
