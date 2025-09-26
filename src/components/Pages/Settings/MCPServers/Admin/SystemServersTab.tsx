import { useState, useEffect } from 'react'
import {
  Table,
  Button,
  Input,
  Select,
  Space,
  Tag,
  Tooltip,
  Switch,
  Drawer,
  Form,
  Card,
  Typography,
  Alert,
  Flex,
  App,
} from 'antd'
import {
  PlusOutlined,
  EditOutlined,
  DeleteOutlined,
  ReloadOutlined,
} from '@ant-design/icons'
import { Stores } from '../../../../../store'
import {
  loadSystemServers,
  createSystemServer,
  updateSystemServer,
  deleteSystemServer,
  startSystemServer,
  stopSystemServer,
  restartSystemServer,
  refreshSystemServers,
  clearAdminMCPErrors,
} from '../../../../../store/admin/mcpServers.ts'
import type {
  MCPServer,
  CreateSystemMCPServerRequest,
} from '../../../../../types/api.ts'

const { Text } = Typography
const { Search } = Input

interface SystemServerFormData {
  name: string
  display_name: string
  description?: string
  transport_type: 'http' | 'process' | 'stdio' | 'sse'
  url?: string
  command?: string
  environment_variables?: Record<string, string>
  enabled: boolean
  is_system: boolean
}

export function SystemServersTab() {
  const { message, modal } = App.useApp()
  const [searchText, setSearchText] = useState('')
  const [statusFilter, setStatusFilter] = useState<string>('all')
  const [drawerVisible, setDrawerVisible] = useState(false)
  const [editingServer, setEditingServer] = useState<MCPServer | null>(null)
  const [form] = Form.useForm<SystemServerFormData>()
  const [operationLoading, setOperationLoading] = useState<
    Record<string, boolean>
  >({})

  const {
    systemServers,
    systemServersLoading,
    systemServersError,
    systemServersInitialized,
  } = Stores.AdminMCPServers

  // Load servers on mount
  useEffect(() => {
    if (!systemServersInitialized) {
      loadSystemServers().catch(console.error)
    }
  }, [systemServersInitialized])

  // Clear error when component mounts
  useEffect(() => {
    if (systemServersError) {
      clearAdminMCPErrors()
    }
  }, [])

  const handleCreateServer = () => {
    setEditingServer(null)
    form.resetFields()
    form.setFieldsValue({
      enabled: true,
      is_system: true,
      transport_type: 'process',
    })
    setDrawerVisible(true)
  }

  const handleEditServer = (server: MCPServer) => {
    setEditingServer(server)
    form.setFieldsValue({
      name: server.name,
      display_name: server.display_name,
      description: server.description,
      transport_type: server.transport_type,
      url: server.url,
      command: server.command,
      environment_variables: server.environment_variables,
      enabled: server.enabled,
      is_system: server.is_system,
    })
    setDrawerVisible(true)
  }

  const handleDeleteServer = (server: MCPServer) => {
    modal.confirm({
      title: 'Delete System Server',
      content: `Are you sure you want to delete "${server.display_name}"? This action cannot be undone.`,
      okType: 'danger',
      onOk: async () => {
        try {
          setOperationLoading(prev => ({
            ...prev,
            [`delete-${server.id}`]: true,
          }))
          await deleteSystemServer(server.id)
          message.success('Server deleted successfully')
        } catch (error) {
          message.error('Failed to delete server')
        } finally {
          setOperationLoading(prev => ({
            ...prev,
            [`delete-${server.id}`]: false,
          }))
        }
      },
    })
  }

  const handleToggleServer = async (server: MCPServer, checked: boolean) => {
    const operation = checked ? 'start' : 'stop'
    try {
      setOperationLoading(prev => ({
        ...prev,
        [`${operation}-${server.id}`]: true,
      }))
      if (checked) {
        await startSystemServer(server.id)
        message.success('Server started successfully')
      } else {
        await stopSystemServer(server.id)
        message.success('Server stopped successfully')
      }
    } catch (error) {
      message.error(`Failed to ${operation} server`)
    } finally {
      setOperationLoading(prev => ({
        ...prev,
        [`${operation}-${server.id}`]: false,
      }))
    }
  }

  const handleRestartServer = async (server: MCPServer) => {
    try {
      setOperationLoading(prev => ({ ...prev, [`restart-${server.id}`]: true }))
      await restartSystemServer(server.id)
      message.success('Server restarted successfully')
    } catch (error) {
      message.error('Failed to restart server')
    } finally {
      setOperationLoading(prev => ({
        ...prev,
        [`restart-${server.id}`]: false,
      }))
    }
  }

  // Tool discovery removed - not available in current admin store

  // Server enable/disable removed - not available in current admin store

  const handleRefreshStatus = async () => {
    try {
      await refreshSystemServers()
      message.success('Server status refreshed')
    } catch (error) {
      message.error('Failed to refresh server status')
    }
  }

  const handleSubmit = async (values: SystemServerFormData) => {
    try {
      if (editingServer) {
        await updateSystemServer(editingServer.id, values)
        message.success('Server updated successfully')
      } else {
        const createData = {
          ...values,
          environment_variables: values.environment_variables || {},
        } as CreateSystemMCPServerRequest
        await createSystemServer(createData)
        message.success('Server created successfully')
      }
      setDrawerVisible(false)
      form.resetFields()
    } catch (error) {
      message.error(`Failed to ${editingServer ? 'update' : 'create'} server`)
    }
  }

  // Filter servers based on search and status
  const filteredServers = systemServers.filter(server => {
    const matchesSearch =
      !searchText ||
      server.display_name.toLowerCase().includes(searchText.toLowerCase()) ||
      server.name.toLowerCase().includes(searchText.toLowerCase()) ||
      (server.description &&
        server.description.toLowerCase().includes(searchText.toLowerCase()))

    const matchesStatus =
      statusFilter === 'all' ||
      (statusFilter === 'active' && server.is_active) ||
      (statusFilter === 'inactive' && !server.is_active) ||
      (statusFilter === 'enabled' && server.enabled) ||
      (statusFilter === 'disabled' && !server.enabled) ||
      (statusFilter === 'error' && server.status === 'error')

    return matchesSearch && matchesStatus
  })

  const getStatusColor = (server: MCPServer) => {
    if (!server.enabled) return 'default'
    if (server.is_active) return 'success'
    if (server.status === 'error') return 'error'
    return 'processing'
  }

  const getStatusText = (server: MCPServer) => {
    if (!server.enabled) return 'Disabled'
    if (server.is_active) return 'Running'
    if (server.status === 'error') return 'Error'
    return 'Stopped'
  }

  const columns = [
    {
      title: 'Server',
      key: 'server',
      render: (server: MCPServer) => (
        <div>
          <div className="flex items-center gap-2">
            <Text strong>{server.display_name}</Text>
            {server.is_system && <Tag color="blue">System</Tag>}
          </div>
          <Text type="secondary" className="text-sm">
            {server.name}
          </Text>
          {server.description && (
            <Text type="secondary" className="text-xs block">
              {server.description}
            </Text>
          )}
        </div>
      ),
    },
    {
      title: 'Status',
      key: 'status',
      render: (server: MCPServer) => (
        <Tag color={getStatusColor(server)}>{getStatusText(server)}</Tag>
      ),
    },
    {
      title: 'Transport',
      dataIndex: 'transport_type',
      key: 'transport_type',
      render: (type: string) => <Tag>{type.toUpperCase()}</Tag>,
    },
    {
      title: 'Tools',
      key: 'tools',
      render: (server: MCPServer) => <Text>{server.tool_count || 0}</Text>,
    },
    {
      title: 'Status',
      key: 'enabled',
      render: (server: MCPServer) => (
        <Tag color={server.enabled ? 'green' : 'default'}>
          {server.enabled ? 'Enabled' : 'Disabled'}
        </Tag>
      ),
    },
    {
      title: 'Running',
      key: 'running',
      render: (server: MCPServer) => (
        <Switch
          checked={server.is_active}
          onChange={checked => handleToggleServer(server, checked)}
          loading={
            operationLoading[`start-${server.id}`] ||
            operationLoading[`stop-${server.id}`]
          }
          disabled={!server.enabled}
          size="small"
        />
      ),
    },
    {
      title: 'Actions',
      key: 'actions',
      render: (server: MCPServer) => (
        <Space size="small">
          <Tooltip title="Edit Server">
            <Button
              type="text"
              icon={<EditOutlined />}
              onClick={() => handleEditServer(server)}
              size="small"
            />
          </Tooltip>
          {server.is_active && (
            <Tooltip title="Restart Server">
              <Button
                type="text"
                icon={<ReloadOutlined />}
                onClick={() => handleRestartServer(server)}
                loading={operationLoading[`restart-${server.id}`]}
                size="small"
              />
            </Tooltip>
          )}
          <Tooltip title="Delete Server">
            <Button
              type="text"
              danger
              icon={<DeleteOutlined />}
              onClick={() => handleDeleteServer(server)}
              loading={operationLoading[`delete-${server.id}`]}
              size="small"
            />
          </Tooltip>
        </Space>
      ),
    },
  ]

  return (
    <div className="space-y-4">
      {/* Header Actions */}
      <Card size="small">
        <Flex justify="space-between" align="center" className="mb-4">
          <div className="flex items-center gap-4">
            <Search
              placeholder="Search servers..."
              value={searchText}
              onChange={e => setSearchText(e.target.value)}
              style={{ width: 300 }}
              allowClear
            />
            <Select
              value={statusFilter}
              onChange={setStatusFilter}
              style={{ width: 120 }}
            >
              <Select.Option value="all">All Status</Select.Option>
              <Select.Option value="active">Active</Select.Option>
              <Select.Option value="inactive">Inactive</Select.Option>
              <Select.Option value="enabled">Enabled</Select.Option>
              <Select.Option value="disabled">Disabled</Select.Option>
              <Select.Option value="error">Error</Select.Option>
            </Select>
          </div>
          <Space>
            <Button
              icon={<ReloadOutlined />}
              onClick={handleRefreshStatus}
              title="Refresh Status"
            >
              Refresh
            </Button>
            <Button
              type="primary"
              icon={<PlusOutlined />}
              onClick={handleCreateServer}
            >
              Create Server
            </Button>
          </Space>
        </Flex>

        {systemServersError && (
          <Alert
            type="error"
            message="Error loading servers"
            description={systemServersError}
            closable
            onClose={clearAdminMCPErrors}
            className="mb-4"
          />
        )}
      </Card>

      {/* Servers Table */}
      <Card>
        <Table
          columns={columns}
          dataSource={filteredServers}
          rowKey="id"
          loading={systemServersLoading}
          pagination={{
            showSizeChanger: true,
            showQuickJumper: true,
            showTotal: (total, range) =>
              `${range[0]}-${range[1]} of ${total} servers`,
          }}
        />
      </Card>

      {/* Create/Edit Server Drawer */}
      <Drawer
        title={editingServer ? 'Edit System Server' : 'Create System Server'}
        open={drawerVisible}
        onClose={() => {
          setDrawerVisible(false)
          form.resetFields()
        }}
        width={600}
        extra={
          <Space>
            <Button onClick={() => setDrawerVisible(false)}>Cancel</Button>
            <Button type="primary" onClick={() => form.submit()}>
              {editingServer ? 'Update' : 'Create'}
            </Button>
          </Space>
        }
      >
        <Form form={form} layout="vertical" onFinish={handleSubmit}>
          <Form.Item
            name="name"
            label="Server Name"
            rules={[{ required: true, message: 'Server name is required' }]}
          >
            <Input placeholder="unique-server-name" />
          </Form.Item>

          <Form.Item
            name="display_name"
            label="Display Name"
            rules={[{ required: true, message: 'Display name is required' }]}
          >
            <Input placeholder="Human readable name" />
          </Form.Item>

          <Form.Item name="description" label="Description">
            <Input.TextArea placeholder="Optional description" />
          </Form.Item>

          <Form.Item
            name="transport_type"
            label="Transport Type"
            rules={[{ required: true, message: 'Transport type is required' }]}
          >
            <Select>
              <Select.Option value="process">Process</Select.Option>
              <Select.Option value="http">HTTP</Select.Option>
              <Select.Option value="stdio">STDIO</Select.Option>
              <Select.Option value="sse">SSE</Select.Option>
            </Select>
          </Form.Item>

          <Form.Item
            noStyle
            shouldUpdate={(prevValues, currentValues) =>
              prevValues.transport_type !== currentValues.transport_type
            }
          >
            {({ getFieldValue }) => {
              const transportType = getFieldValue('transport_type')

              if (transportType === 'process') {
                return (
                  <Form.Item
                    name="command"
                    label="Command"
                    rules={[
                      {
                        required: true,
                        message: 'Command is required for process transport',
                      },
                    ]}
                  >
                    <Input placeholder="/path/to/executable" />
                  </Form.Item>
                )
              }

              if (transportType === 'http' || transportType === 'websocket') {
                return (
                  <Form.Item
                    name="url"
                    label="URL"
                    rules={[
                      {
                        required: true,
                        message: 'URL is required for HTTP/WebSocket transport',
                      },
                    ]}
                  >
                    <Input placeholder="http://localhost:8080" />
                  </Form.Item>
                )
              }

              return null
            }}
          </Form.Item>

          <div className="flex gap-4">
            <Form.Item name="enabled" valuePropName="checked">
              <div className="flex items-center gap-2">
                <Switch size="small" />
                <span>Enabled</span>
              </div>
            </Form.Item>

            <Form.Item name="is_system" valuePropName="checked">
              <div className="flex items-center gap-2">
                <Switch size="small" />
                <span>System Server</span>
              </div>
            </Form.Item>
          </div>
        </Form>
      </Drawer>
    </div>
  )
}
