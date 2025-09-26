import { useState } from 'react'
import {
  Card,
  Row,
  Col,
  Statistic,
  Table,
  Select,
  DatePicker,
  Button,
  Typography,
  Progress,
  Tag,
  Flex,
  App,
} from 'antd'
import {
  ReloadOutlined,
  DatabaseOutlined as ServerOutlined,
  ToolOutlined,
  PlayCircleOutlined,
  CheckCircleOutlined,
  CloseCircleOutlined,
  ClockCircleOutlined,
} from '@ant-design/icons'
import { Stores } from '../../../../../store'
import {
  getExecutionStats,
  getActiveExecutions,
} from '../../../../../store/mcpExecution.ts'
import type { MCPExecutionLog } from '../../../../../types/api.ts'
import dayjs, { Dayjs } from 'dayjs'

const { Text } = Typography
const { RangePicker } = DatePicker

interface ToolStats {
  toolName: string
  serverId: string
  serverName: string
  totalExecutions: number
  successRate: number
  averageDuration: number
  lastUsed: string
}

interface ServerStats {
  serverId: string
  serverName: string
  totalExecutions: number
  activeTools: number
  successRate: number
  isActive: boolean
}

export function StatisticsTab() {
  const { message } = App.useApp()
  const [dateRange, setDateRange] = useState<[Dayjs | null, Dayjs | null]>([
    dayjs().subtract(30, 'days'),
    dayjs(),
  ])
  const [viewMode, setViewMode] = useState<'overview' | 'tools' | 'servers'>(
    'overview',
  )
  const [refreshing, setRefreshing] = useState(false)

  const { executionLogs } = Stores.MCPExecution
  const { systemServers } = Stores.AdminMCPServers

  // Calculate statistics
  const stats = getExecutionStats()
  const activeExecutions = getActiveExecutions()

  // Filter logs by date range
  const filteredLogs = executionLogs.filter(log => {
    const logDate = dayjs(log.started_at)
    const startDate = dateRange[0]
    const endDate = dateRange[1]

    if (startDate && logDate.isBefore(startDate)) return false
    if (endDate && logDate.isAfter(endDate)) return false

    return true
  })

  // Calculate tool statistics
  const toolStats: ToolStats[] = []
  const toolMap = new Map<string, MCPExecutionLog[]>()

  filteredLogs.forEach(log => {
    const key = `${log.server_id}-${log.tool_name}`
    if (!toolMap.has(key)) {
      toolMap.set(key, [])
    }
    toolMap.get(key)!.push(log)
  })

  toolMap.forEach((logs, key) => {
    const [serverId, toolName] = key.split('-')
    const server = systemServers.find((s: any) => s.id === serverId)
    const completedLogs = logs.filter(l => l.status === 'completed')
    const totalDuration = logs
      .filter(l => l.duration_ms)
      .reduce((sum, l) => sum + (l.duration_ms || 0), 0)

    toolStats.push({
      toolName,
      serverId,
      serverName: server?.display_name || serverId,
      totalExecutions: logs.length,
      successRate:
        logs.length > 0 ? (completedLogs.length / logs.length) * 100 : 0,
      averageDuration: logs.length > 0 ? totalDuration / logs.length : 0,
      lastUsed: logs.length > 0 ? logs[logs.length - 1].started_at : '',
    })
  })

  // Calculate server statistics
  const serverStats: ServerStats[] = systemServers.map(server => {
    const serverLogs = filteredLogs.filter(log => log.server_id === server.id)
    const completedLogs = serverLogs.filter(l => l.status === 'completed')
    const uniqueTools = new Set(serverLogs.map(l => l.tool_name)).size

    return {
      serverId: server.id,
      serverName: server.display_name,
      totalExecutions: serverLogs.length,
      activeTools: uniqueTools,
      successRate:
        serverLogs.length > 0
          ? (completedLogs.length / serverLogs.length) * 100
          : 0,
      isActive: server.is_active,
    }
  })

  const handleRefresh = async () => {
    setRefreshing(true)
    try {
      // In real implementation, this would refresh all relevant data
      await new Promise(resolve => setTimeout(resolve, 1000))
      message.success('Statistics refreshed')
    } catch (error) {
      message.error('Failed to refresh statistics')
    } finally {
      setRefreshing(false)
    }
  }

  const formatDuration = (durationMs: number) => {
    if (durationMs < 1000) return `${durationMs}ms`
    if (durationMs < 60000) return `${(durationMs / 1000).toFixed(1)}s`
    return `${(durationMs / 60000).toFixed(1)}min`
  }

  const getSuccessRateColor = (rate: number) => {
    if (rate >= 90) return '#52c41a'
    if (rate >= 70) return '#faad14'
    return '#f5222d'
  }

  const toolColumns = [
    {
      title: 'Tool',
      key: 'tool',
      render: (tool: ToolStats) => (
        <div>
          <div className="flex items-center gap-2">
            <ToolOutlined />
            <Text strong>{tool.toolName}</Text>
          </div>
          <Text type="secondary" className="text-sm">
            {tool.serverName}
          </Text>
        </div>
      ),
    },
    {
      title: 'Executions',
      dataIndex: 'totalExecutions',
      key: 'executions',
      sorter: (a: ToolStats, b: ToolStats) =>
        a.totalExecutions - b.totalExecutions,
    },
    {
      title: 'Success Rate',
      key: 'successRate',
      render: (tool: ToolStats) => (
        <div>
          <Progress
            percent={tool.successRate}
            size="small"
            strokeColor={getSuccessRateColor(tool.successRate)}
          />
        </div>
      ),
      sorter: (a: ToolStats, b: ToolStats) => a.successRate - b.successRate,
    },
    {
      title: 'Avg Duration',
      key: 'avgDuration',
      render: (tool: ToolStats) => formatDuration(tool.averageDuration),
      sorter: (a: ToolStats, b: ToolStats) =>
        a.averageDuration - b.averageDuration,
    },
    {
      title: 'Last Used',
      key: 'lastUsed',
      render: (tool: ToolStats) =>
        tool.lastUsed ? dayjs(tool.lastUsed).format('MMM D, HH:mm') : 'Never',
      sorter: (a: ToolStats, b: ToolStats) =>
        dayjs(a.lastUsed).unix() - dayjs(b.lastUsed).unix(),
    },
  ]

  const serverColumns = [
    {
      title: 'Server',
      key: 'server',
      render: (server: ServerStats) => (
        <div>
          <div className="flex items-center gap-2">
            <ServerOutlined />
            <Text strong>{server.serverName}</Text>
            <Tag color={server.isActive ? 'green' : 'default'}>
              {server.isActive ? 'Active' : 'Inactive'}
            </Tag>
          </div>
        </div>
      ),
    },
    {
      title: 'Executions',
      dataIndex: 'totalExecutions',
      key: 'executions',
      sorter: (a: ServerStats, b: ServerStats) =>
        a.totalExecutions - b.totalExecutions,
    },
    {
      title: 'Active Tools',
      dataIndex: 'activeTools',
      key: 'activeTools',
      sorter: (a: ServerStats, b: ServerStats) => a.activeTools - b.activeTools,
    },
    {
      title: 'Success Rate',
      key: 'successRate',
      render: (server: ServerStats) => (
        <div>
          <Progress
            percent={server.successRate}
            size="small"
            strokeColor={getSuccessRateColor(server.successRate)}
          />
        </div>
      ),
      sorter: (a: ServerStats, b: ServerStats) => a.successRate - b.successRate,
    },
  ]

  return (
    <div className="space-y-4">
      {/* Filters and Actions */}
      <Card size="small">
        <Flex justify="space-between" align="center">
          <div className="flex items-center gap-4">
            <RangePicker
              value={dateRange}
              onChange={dates =>
                setDateRange(dates as [Dayjs | null, Dayjs | null])
              }
              presets={[
                {
                  label: 'Last 7 days',
                  value: [dayjs().subtract(7, 'days'), dayjs()],
                },
                {
                  label: 'Last 30 days',
                  value: [dayjs().subtract(30, 'days'), dayjs()],
                },
                {
                  label: 'Last 90 days',
                  value: [dayjs().subtract(90, 'days'), dayjs()],
                },
              ]}
            />
            <Select
              value={viewMode}
              onChange={setViewMode}
              style={{ width: 150 }}
            >
              <Select.Option value="overview">Overview</Select.Option>
              <Select.Option value="tools">Tools</Select.Option>
              <Select.Option value="servers">Servers</Select.Option>
            </Select>
          </div>
          <Button
            icon={<ReloadOutlined />}
            onClick={handleRefresh}
            loading={refreshing}
          >
            Refresh
          </Button>
        </Flex>
      </Card>

      {viewMode === 'overview' && (
        <>
          {/* Overview Statistics */}
          <Row gutter={16}>
            <Col span={6}>
              <Card>
                <Statistic
                  title="Total Executions"
                  value={stats.total}
                  prefix={<PlayCircleOutlined />}
                  valueStyle={{ color: '#1890ff' }}
                />
              </Card>
            </Col>
            <Col span={6}>
              <Card>
                <Statistic
                  title="Success Rate"
                  value={
                    stats.total > 0
                      ? ((stats.completed / stats.total) * 100).toFixed(1)
                      : 0
                  }
                  suffix="%"
                  prefix={<CheckCircleOutlined />}
                  valueStyle={{ color: '#52c41a' }}
                />
              </Card>
            </Col>
            <Col span={6}>
              <Card>
                <Statistic
                  title="Failed"
                  value={stats.failed}
                  prefix={<CloseCircleOutlined />}
                  valueStyle={{ color: '#f5222d' }}
                />
              </Card>
            </Col>
            <Col span={6}>
              <Card>
                <Statistic
                  title="Currently Running"
                  value={stats.running + stats.pending}
                  prefix={<ClockCircleOutlined />}
                  valueStyle={{ color: '#faad14' }}
                />
              </Card>
            </Col>
          </Row>

          {/* Status Distribution */}
          <Card title="Execution Status Distribution">
            <Row gutter={16}>
              <Col span={4}>
                <Card size="small">
                  <Statistic
                    title="Completed"
                    value={stats.completed}
                    valueStyle={{ color: '#52c41a', fontSize: '20px' }}
                  />
                </Card>
              </Col>
              <Col span={4}>
                <Card size="small">
                  <Statistic
                    title="Failed"
                    value={stats.failed}
                    valueStyle={{ color: '#f5222d', fontSize: '20px' }}
                  />
                </Card>
              </Col>
              <Col span={4}>
                <Card size="small">
                  <Statistic
                    title="Running"
                    value={stats.running}
                    valueStyle={{ color: '#1890ff', fontSize: '20px' }}
                  />
                </Card>
              </Col>
              <Col span={4}>
                <Card size="small">
                  <Statistic
                    title="Pending"
                    value={stats.pending}
                    valueStyle={{ color: '#faad14', fontSize: '20px' }}
                  />
                </Card>
              </Col>
              <Col span={4}>
                <Card size="small">
                  <Statistic
                    title="Cancelled"
                    value={stats.cancelled}
                    valueStyle={{ color: '#d9d9d9', fontSize: '20px' }}
                  />
                </Card>
              </Col>
              <Col span={4}>
                <Card size="small">
                  <Statistic
                    title="Timeout"
                    value={stats.timeout}
                    valueStyle={{ color: '#722ed1', fontSize: '20px' }}
                  />
                </Card>
              </Col>
            </Row>
          </Card>

          {/* Active Executions */}
          {activeExecutions.length > 0 && (
            <Card title="Currently Active Executions">
              <div className="space-y-2">
                {activeExecutions.map(execution => (
                  <div
                    key={execution.execution_id}
                    className="flex items-center justify-between p-2 bg-gray-50 rounded"
                  >
                    <div>
                      <Text strong>
                        {(execution as any).tool_name || 'Unknown Tool'}
                      </Text>
                      <Text type="secondary" className="ml-2">
                        {systemServers.find(
                          s => s.id === (execution as any).server_id,
                        )?.display_name || 'Unknown Server'}
                      </Text>
                    </div>
                    <Tag color="processing">
                      {execution.status.toUpperCase()}
                    </Tag>
                  </div>
                ))}
              </div>
            </Card>
          )}
        </>
      )}

      {viewMode === 'tools' && (
        <Card title="Tool Statistics">
          <Table
            columns={toolColumns}
            dataSource={toolStats}
            rowKey={tool => `${tool.serverId}-${tool.toolName}`}
            pagination={{
              showSizeChanger: true,
              showQuickJumper: true,
              showTotal: (total, range) =>
                `${range[0]}-${range[1]} of ${total} tools`,
            }}
          />
        </Card>
      )}

      {viewMode === 'servers' && (
        <Card title="Server Statistics">
          <Table
            columns={serverColumns}
            dataSource={serverStats}
            rowKey="serverId"
            pagination={{
              showSizeChanger: true,
              showQuickJumper: true,
              showTotal: (total, range) =>
                `${range[0]}-${range[1]} of ${total} servers`,
            }}
          />
        </Card>
      )}
    </div>
  )
}
