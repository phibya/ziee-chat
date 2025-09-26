import { useState, useEffect } from 'react'
import {
  Table,
  Button,
  Input,
  Select,
  Card,
  Typography,
  Alert,
  Flex,
  Tag,
  App,
} from 'antd'
import { ReloadOutlined } from '@ant-design/icons'
import { Stores } from '../../../../../store'
import {
  loadExecutionLogs,
  refreshExecutionLogs,
  clearExecutionError,
  getExecutionStats,
} from '../../../../../store/mcpExecution.ts'
import type {
  MCPExecutionLog,
  MCPExecutionStatus,
} from '../../../../../types/api.ts'
import dayjs from 'dayjs'

const { Text } = Typography
const { Search } = Input

export function SimpleExecutionLogsTab() {
  const { message } = App.useApp()
  const [searchText, setSearchText] = useState('')
  const [statusFilter, setStatusFilter] = useState<MCPExecutionStatus | 'all'>(
    'all',
  )

  const {
    executionLogs,
    executionLogsLoading,
    executionLogsError,
    executionLogsInitialized,
  } = Stores.MCPExecution

  const { systemServers } = Stores.AdminMCPServers

  // Load execution logs on mount
  useEffect(() => {
    if (!executionLogsInitialized) {
      loadExecutionLogs().catch(console.error)
    }
  }, [executionLogsInitialized])

  // Clear error when component mounts
  useEffect(() => {
    if (executionLogsError) {
      clearExecutionError()
    }
  }, [])

  const handleRefreshLogs = async () => {
    try {
      await refreshExecutionLogs()
      message.success('Execution logs refreshed')
    } catch (error) {
      message.error('Failed to refresh execution logs')
    }
  }

  // Get execution statistics
  const stats = getExecutionStats()

  // Filter logs
  let filteredLogs = executionLogs

  if (searchText) {
    filteredLogs = filteredLogs.filter(
      log =>
        log.tool_name.toLowerCase().includes(searchText.toLowerCase()) ||
        log.server_id.toLowerCase().includes(searchText.toLowerCase()),
    )
  }

  if (statusFilter !== 'all') {
    filteredLogs = filteredLogs.filter(log => log.status === statusFilter)
  }

  const getStatusColor = (status: MCPExecutionStatus) => {
    switch (status) {
      case 'completed':
        return 'success'
      case 'failed':
        return 'error'
      case 'cancelled':
        return 'default'
      case 'running':
        return 'processing'
      case 'pending':
        return 'warning'
      case 'timeout':
        return 'error'
      default:
        return 'default'
    }
  }

  const formatDuration = (durationMs?: number) => {
    if (!durationMs) return '-'
    if (durationMs < 1000) return `${durationMs}ms`
    if (durationMs < 60000) return `${(durationMs / 1000).toFixed(1)}s`
    return `${(durationMs / 60000).toFixed(1)}min`
  }

  const columns = [
    {
      title: 'Execution ID',
      key: 'id',
      render: (log: MCPExecutionLog) => (
        <Text code className="text-xs">
          {log.id.substring(0, 8)}...
        </Text>
      ),
    },
    {
      title: 'Tool',
      key: 'tool',
      render: (log: MCPExecutionLog) => (
        <div>
          <Text strong className="text-sm">
            {log.tool_name}
          </Text>
          <Text type="secondary" className="text-xs block">
            {systemServers.find(s => s.id === log.server_id)?.display_name ||
              log.server_id}
          </Text>
        </div>
      ),
    },
    {
      title: 'Status',
      key: 'status',
      render: (log: MCPExecutionLog) => (
        <Tag color={getStatusColor(log.status)}>{log.status.toUpperCase()}</Tag>
      ),
    },
    {
      title: 'Duration',
      key: 'duration',
      render: (log: MCPExecutionLog) => (
        <Text className="text-sm">{formatDuration(log.duration_ms)}</Text>
      ),
    },
    {
      title: 'Started',
      key: 'started',
      render: (log: MCPExecutionLog) => (
        <Text className="text-sm">
          {dayjs(log.started_at).format('MMM D, HH:mm:ss')}
        </Text>
      ),
    },
  ]

  return (
    <div className="space-y-4">
      {/* Statistics Cards */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <Card size="small">
          <div className="text-center">
            <div className="text-2xl font-bold text-blue-600">
              {stats.total}
            </div>
            <div className="text-sm text-gray-500">Total Executions</div>
          </div>
        </Card>
        <Card size="small">
          <div className="text-center">
            <div className="text-2xl font-bold text-green-600">
              {stats.completed}
            </div>
            <div className="text-sm text-gray-500">Completed</div>
          </div>
        </Card>
        <Card size="small">
          <div className="text-center">
            <div className="text-2xl font-bold text-red-600">
              {stats.failed}
            </div>
            <div className="text-sm text-gray-500">Failed</div>
          </div>
        </Card>
        <Card size="small">
          <div className="text-center">
            <div className="text-2xl font-bold text-orange-600">
              {stats.running + stats.pending}
            </div>
            <div className="text-sm text-gray-500">Active</div>
          </div>
        </Card>
      </div>

      {/* Filters and Actions */}
      <Card size="small">
        <Flex justify="space-between" align="center" className="mb-4">
          <div className="flex items-center gap-4 flex-wrap">
            <Search
              placeholder="Search executions..."
              value={searchText}
              onChange={e => setSearchText(e.target.value)}
              style={{ width: 250 }}
              allowClear
            />
            <Select
              value={statusFilter}
              onChange={setStatusFilter}
              style={{ width: 120 }}
            >
              <Select.Option value="all">All Status</Select.Option>
              <Select.Option value="completed">Completed</Select.Option>
              <Select.Option value="failed">Failed</Select.Option>
              <Select.Option value="running">Running</Select.Option>
              <Select.Option value="pending">Pending</Select.Option>
              <Select.Option value="cancelled">Cancelled</Select.Option>
              <Select.Option value="timeout">Timeout</Select.Option>
            </Select>
          </div>
          <Button
            icon={<ReloadOutlined />}
            onClick={handleRefreshLogs}
            loading={executionLogsLoading}
          >
            Refresh
          </Button>
        </Flex>

        {executionLogsError && (
          <Alert
            type="error"
            message="Error loading execution logs"
            description={executionLogsError}
            closable
            onClose={clearExecutionError}
            className="mb-4"
          />
        )}
      </Card>

      {/* Execution Logs Table */}
      <Card>
        <Table
          columns={columns}
          dataSource={filteredLogs}
          rowKey="id"
          loading={executionLogsLoading}
          pagination={{
            showSizeChanger: true,
            showQuickJumper: true,
            showTotal: (total, range) =>
              `${range[0]}-${range[1]} of ${total} executions`,
          }}
        />
      </Card>
    </div>
  )
}

export { SimpleExecutionLogsTab as ExecutionLogsTab }
