import { useState, useEffect } from 'react'
import {
  App,
  Button,
  Card,
  Tag,
  Typography,
  Tooltip,
  Switch,
  Flex,
  Dropdown,
  List,
  Empty,
  Badge,
} from 'antd'
import {
  EditOutlined,
  DeleteOutlined,
  PlayCircleOutlined,
  StopOutlined,
  MoreOutlined,
  ToolOutlined,
  LinkOutlined,
  DownOutlined,
  UpOutlined,
  ReloadOutlined,
} from '@ant-design/icons'
import type { MCPServer, MCPToolWithServer } from '../../../../types/api'
import {
  deleteMCPServer,
  startMCPServer,
  stopMCPServer,
  restartMCPServer,
  discoverServerTools,
  getServerTools,
} from '../../../../store/mcp'
import { openMCPServerDrawer } from '../../../../store/ui/mcpDrawers'
import { ToolTestingModal } from './ToolTestingModal'

const { Text } = Typography

interface MCPServerCardProps {
  server: MCPServer
}

export function MCPServerCard({ server }: MCPServerCardProps) {
  const { message, modal } = App.useApp()
  const [showTools, setShowTools] = useState(false)
  const [tools, setTools] = useState<MCPToolWithServer[]>([])
  const [loadingTools, setLoadingTools] = useState(false)
  const [testingTool, setTestingTool] = useState<MCPToolWithServer | null>(null)
  const [operationLoading, setOperationLoading] = useState<string | null>(null)

  // Load tools when server is active and tools section is expanded
  const loadServerTools = async () => {
    if (!server.is_active || loadingTools) return

    setLoadingTools(true)
    try {
      const serverTools = await getServerTools(server.id)
      // Convert MCPTool[] to MCPToolWithServer[] by adding server info
      const toolsWithServer: MCPToolWithServer[] = serverTools.map(tool => ({
        ...tool,
        server_id: server.id,
        server_name: server.name,
        server_display_name: server.display_name,
        is_system: server.is_system,
        transport_type: server.transport_type,
      }))
      setTools(toolsWithServer)
    } catch (error) {
      console.error('Failed to load server tools:', error)
      message.error('Failed to load tools for this server')
    } finally {
      setLoadingTools(false)
    }
  }

  // Auto-load tools when showing tools for an active server
  useEffect(() => {
    if (showTools && server.is_active && tools.length === 0) {
      loadServerTools()
    }
  }, [showTools, server.is_active])

  const handleEdit = () => {
    openMCPServerDrawer(server, 'edit')
  }

  const handleDelete = () => {
    modal.confirm({
      title: 'Delete MCP Server',
      content: `Are you sure you want to delete "${server.display_name}"? This action cannot be undone.`,
      okType: 'danger',
      onOk: async () => {
        try {
          await deleteMCPServer(server.id)
          message.success('Server deleted successfully')
        } catch (error) {
          message.error('Failed to delete server')
        }
      },
    })
  }

  const handleToggleStatus = async (checked: boolean) => {
    const operation = checked ? 'start' : 'stop'
    setOperationLoading(operation)

    try {
      if (checked) {
        await startMCPServer(server.id)
      } else {
        await stopMCPServer(server.id)
      }
      message.success(`Server ${checked ? 'started' : 'stopped'} successfully`)

      // Clear tools when server is stopped
      if (!checked) {
        setTools([])
        setShowTools(false)
      }
    } catch (error) {
      message.error(`Failed to ${checked ? 'start' : 'stop'} server`)
    } finally {
      setOperationLoading(null)
    }
  }

  const handleServerAction = async (action: 'restart') => {
    setOperationLoading(action)

    try {
      await restartMCPServer(server.id)
      message.success('Server restarted successfully')

      // Refresh tools if they were showing
      if (showTools) {
        setTools([])
        setTimeout(loadServerTools, 1000) // Wait a bit for server to start
      }
    } catch (error) {
      message.error('Failed to restart server')
    } finally {
      setOperationLoading(null)
    }
  }

  const handleToggleTools = () => {
    const newShowTools = !showTools
    setShowTools(newShowTools)

    // Load tools if server is active and we're showing tools
    if (newShowTools && server.is_active && tools.length === 0) {
      loadServerTools()
    }
  }

  const handleDiscoverTools = async () => {
    setOperationLoading('discover')

    try {
      await discoverServerTools(server.id)
      message.success('Tool discovery completed successfully')

      // Refresh tools if they're showing
      if (showTools) {
        loadServerTools()
      }
    } catch (error) {
      message.error('Failed to discover tools')
    } finally {
      setOperationLoading(null)
    }
  }

  const handleRefreshTools = () => {
    if (server.is_active) {
      loadServerTools()
    }
  }

  const handleTestTool = (tool: MCPToolWithServer) => {
    setTestingTool(tool)
  }

  const handleCloseTestModal = () => {
    setTestingTool(null)
  }

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

  const menuItems = [
    {
      key: 'edit',
      icon: <EditOutlined />,
      label: 'Edit',
      onClick: handleEdit,
    },
    ...(server.is_active
      ? [
          {
            key: 'restart-server',
            icon: <ReloadOutlined />,
            label: 'Restart Server',
            onClick: () => handleServerAction('restart'),
            disabled: operationLoading === 'restart',
          },
          {
            key: 'refresh-tools',
            icon: <ToolOutlined />,
            label: 'Discover Tools',
            onClick: handleDiscoverTools,
            disabled: operationLoading === 'discover',
          },
        ]
      : []),
    {
      key: 'toggle-tools',
      icon: showTools ? <UpOutlined /> : <DownOutlined />,
      label: showTools ? 'Hide Tools' : 'Show Tools',
      onClick: handleToggleTools,
      disabled: !server.is_active,
    },
    { type: 'divider' as const },
    ...(server.is_system
      ? []
      : [
          {
            key: 'delete',
            icon: <DeleteOutlined />,
            label: 'Delete',
            onClick: handleDelete,
            danger: true,
          },
        ]),
  ]

  return (
    <Card
      hoverable
      className="cursor-pointer relative group hover:!shadow-md transition-shadow h-full"
      actions={[
        <Tooltip
          title={server.is_active ? 'Stop Server' : 'Start Server'}
          key="toggle"
        >
          <Switch
            checked={server.is_active}
            onChange={handleToggleStatus}
            loading={
              operationLoading === 'start' || operationLoading === 'stop'
            }
            checkedChildren={<PlayCircleOutlined />}
            unCheckedChildren={<StopOutlined />}
          />
        </Tooltip>,
        <Tooltip title="Edit Server" key="edit">
          <Button type="text" icon={<EditOutlined />} onClick={handleEdit} />
        </Tooltip>,
        <Dropdown menu={{ items: menuItems }} trigger={['click']} key="menu">
          <Button type="text" icon={<MoreOutlined />} />
        </Dropdown>,
      ]}
    >
      <div className="flex items-start gap-3 flex-wrap">
        <div className="flex-1">
          <div className="flex items-center gap-2 mb-2 flex-wrap">
            <div className="flex-1 min-w-48">
              <Flex className="gap-2 items-center flex-wrap">
                <ToolOutlined />
                <Text className="font-medium">{server.display_name}</Text>
                {server.is_system && <Tag color="blue">System</Tag>}
                <Tag color={getStatusColor(server)}>
                  {getStatusText(server)}
                </Tag>
                {server.tool_count && server.tool_count > 0 && (
                  <Tooltip
                    title={
                      server.is_active
                        ? 'Click menu to view tools'
                        : 'Start server to view tools'
                    }
                  >
                    <Tag
                      color="cyan"
                      className={server.is_active ? 'cursor-pointer' : ''}
                      onClick={server.is_active ? handleToggleTools : undefined}
                    >
                      {server.tool_count} tools
                    </Tag>
                  </Tooltip>
                )}
              </Flex>
            </div>
          </div>

          <div className="mb-3">
            <Text type="secondary" className="text-sm">
              {server.description || 'No description'}
            </Text>
          </div>

          <div className="flex items-center gap-2 text-xs text-gray-500 flex-wrap">
            <Flex className="gap-2 items-center">
              <LinkOutlined />
              <Text type="secondary" className="text-xs">
                {server.transport_type.toUpperCase()}
              </Text>
            </Flex>
            {server.url && (
              <Text type="secondary" className="text-xs truncate">
                {server.url}
              </Text>
            )}
            {server.command && (
              <Text type="secondary" className="text-xs truncate">
                {server.command}
              </Text>
            )}
          </div>

          {/* Tools Section - Only show when server is active */}
          {server.is_active && showTools && (
            <div className="mt-4 pt-4 border-t border-gray-200">
              <div className="flex items-center justify-between mb-3">
                <Text strong className="text-sm flex items-center gap-2">
                  <ToolOutlined />
                  Available Tools
                  {tools.length > 0 && (
                    <Badge
                      count={tools.length}
                      showZero
                      style={{ backgroundColor: '#52c41a' }}
                    />
                  )}
                </Text>
                <Button
                  type="text"
                  size="small"
                  icon={<ReloadOutlined />}
                  onClick={handleRefreshTools}
                  loading={loadingTools}
                />
              </div>

              {loadingTools ? (
                <div className="text-center py-4">
                  <Text type="secondary">Loading tools...</Text>
                </div>
              ) : tools.length === 0 ? (
                <Empty
                  image={Empty.PRESENTED_IMAGE_SIMPLE}
                  description="No tools discovered"
                  style={{ margin: '16px 0' }}
                />
              ) : (
                <div className="max-h-32 overflow-y-auto">
                  <List
                    size="small"
                    dataSource={tools}
                    renderItem={tool => (
                      <List.Item className="px-0 py-1">
                        <div className="w-full">
                          <div className="flex items-center justify-between">
                            <Text className="font-medium text-sm">
                              {tool.tool_name}
                            </Text>
                            <div className="flex items-center gap-2">
                              <Badge
                                count={tool.usage_count}
                                showZero
                                style={{
                                  backgroundColor: '#f0f0f0',
                                  color: '#666',
                                }}
                              />
                              <Button
                                type="link"
                                size="small"
                                className="p-0 h-auto text-xs"
                                onClick={() => handleTestTool(tool)}
                              >
                                Test
                              </Button>
                            </div>
                          </div>
                          {tool.tool_description && (
                            <Text
                              type="secondary"
                              className="text-xs block mt-1"
                            >
                              {tool.tool_description.length > 80
                                ? `${tool.tool_description.substring(0, 80)}...`
                                : tool.tool_description}
                            </Text>
                          )}
                        </div>
                      </List.Item>
                    )}
                  />
                </div>
              )}
            </div>
          )}

          {/* Show tools toggle button for inactive servers with tools */}
          {!server.is_active && server.tool_count && server.tool_count > 0 && (
            <div className="mt-4 pt-4 border-t border-gray-200">
              <Text
                type="secondary"
                className="text-sm flex items-center gap-2"
              >
                <ToolOutlined />
                {server.tool_count} tools available when server is running
              </Text>
            </div>
          )}
        </div>
      </div>

      {/* Tool Testing Modal */}
      {testingTool && (
        <ToolTestingModal
          tool={testingTool}
          server={server}
          open={!!testingTool}
          onClose={handleCloseTestModal}
        />
      )}
    </Card>
  )
}
