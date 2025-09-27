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
  List,
  Empty,
  Badge,
} from 'antd'
import {
  EditOutlined,
  ToolOutlined,
  ReloadOutlined,
  CheckOutlined,
  PlayCircleOutlined,
  StopOutlined,
} from '@ant-design/icons'
import type { MCPServer, MCPToolWithServer } from '../../../../types/api'
import {
  startMCPServer,
  stopMCPServer,
  getServerTools,
  setToolGlobalApproval,
  removeToolGlobalApproval,
  updateMCPServer,
} from '../../../../store/mcp'
import { openMCPServerDrawer } from '../../../../store/ui/mcpDrawers'
import { ToolTestingModal } from './ToolTestingModal'

const { Text } = Typography

interface MCPServerCardProps {
  server: MCPServer
  isEditable?: boolean
}

export function MCPServerCard({
  server,
  isEditable = true,
}: MCPServerCardProps) {
  const { message } = App.useApp()
  const [showTools, setShowTools] = useState(false)
  const [tools, setTools] = useState<MCPToolWithServer[]>([])
  const [loadingTools, setLoadingTools] = useState(false)
  const [testingTool, setTestingTool] = useState<MCPToolWithServer | null>(null)
  const [operationLoading, setOperationLoading] = useState<string | null>(null)
  const [enableLoading, setEnableLoading] = useState(false)

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
    if (server.is_system) {
      openMCPServerDrawer(server, 'edit-system')
    } else {
      openMCPServerDrawer(server, 'edit')
    }
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

  const handleToggleTools = () => {
    const newShowTools = !showTools
    setShowTools(newShowTools)

    // Load tools if server is active and we're showing tools
    if (newShowTools && server.is_active && tools.length === 0) {
      loadServerTools()
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

  const handleToggleAutoApprove = async (
    tool: MCPToolWithServer,
    autoApprove: boolean,
  ) => {
    try {
      if (autoApprove) {
        await setToolGlobalApproval(tool.server_id, tool.tool_name, {
          auto_approve: true,
        })
        message.success(`Auto-approve enabled for ${tool.tool_name}`)
      } else {
        await removeToolGlobalApproval(tool.server_id, tool.tool_name)
        message.success(`Auto-approve disabled for ${tool.tool_name}`)
      }

      // Update the local tool state to reflect the change
      setTools(prevTools =>
        prevTools.map(t =>
          t.server_id === tool.server_id && t.tool_name === tool.tool_name
            ? { ...t, global_auto_approve: autoApprove ? true : undefined }
            : t,
        ),
      )
    } catch (error) {
      message.error(
        `Failed to ${autoApprove ? 'enable' : 'disable'} auto-approve for ${tool.tool_name}`,
      )
    }
  }

  const handleToggleEnable = async (enabled: boolean) => {
    setEnableLoading(true)
    try {
      await updateMCPServer(server.id, {
        enabled,
      })
      message.success(`Server ${enabled ? 'enabled' : 'disabled'} successfully`)
    } catch (error) {
      console.error('Failed to toggle server enable state:', error)
      message.error(`Failed to ${enabled ? 'enable' : 'disable'} server`)
    } finally {
      setEnableLoading(false)
    }
  }

  return (
    <>
      <Card>
        <div className="flex items-start gap-3 flex-wrap">
          {/* Server Info */}
          <div className="flex-1">
            <div className="flex items-center gap-2 mb-2 flex-wrap">
              <div className="flex-1 min-w-48">
                <Flex className="gap-2 items-center">
                  <ToolOutlined />
                  <Text className="font-medium">{server.display_name}</Text>
                  {!isEditable && server.is_system && (
                    <Tag color="blue">System</Tag>
                  )}
                  {server.status === 'error' && <Tag color="red">Error</Tag>}
                  {/* Show enabled/disabled status tag for all system servers */}
                  {(server.tool_count ?? 0) > 0 && (
                    <Tag color="cyan">{server.tool_count} tools</Tag>
                  )}
                </Flex>
              </div>
              <div className="flex gap-1 items-center justify-end">
                {/* Enable/disable switch for system servers */}
                {isEditable && (
                  <Tooltip
                    title={server.enabled ? 'Disable Server' : 'Enable Server'}
                  >
                    <Switch
                      checked={server.enabled}
                      onChange={handleToggleEnable}
                      loading={enableLoading}
                    />
                  </Tooltip>
                )}
                {/* Show controls based on isEditable prop */}
                {isEditable && (
                  <>
                    <Tooltip
                      title={server.is_active ? 'Stop Server' : 'Start Server'}
                    >
                      <Button
                        type={server.is_active ? 'default' : 'primary'}
                        icon={
                          server.is_active ? (
                            <StopOutlined />
                          ) : (
                            <PlayCircleOutlined />
                          )
                        }
                        onClick={e => {
                          e.stopPropagation()
                          handleToggleStatus(!server.is_active)
                        }}
                        loading={
                          operationLoading === 'start' ||
                          operationLoading === 'stop'
                        }
                      >
                        {server.is_active ? 'Stop' : 'Start'}
                      </Button>
                    </Tooltip>
                    <Button
                      icon={<EditOutlined />}
                      onClick={e => {
                        e.stopPropagation()
                        handleEdit()
                      }}
                    >
                      Edit
                    </Button>
                  </>
                )}
                <Button
                  type={showTools ? 'primary' : 'default'}
                  icon={<ToolOutlined />}
                  onClick={e => {
                    e.stopPropagation()
                    handleToggleTools()
                  }}
                  disabled={!server.is_active}
                >
                  Tools
                </Button>
              </div>
            </div>

            <div>
              <Text type="secondary" className="text-sm mb-2 block">
                {server.description || 'No description'}
              </Text>

              {/* Transport Information */}
              <div className="mb-2">
                <Text type="secondary" className="text-xs mr-2">
                  Transport:
                </Text>
                <Tag color="default" className="text-xs">
                  {server.transport_type.toUpperCase()}
                </Tag>
                {server.url && (
                  <Text type="secondary" className="text-xs ml-2 truncate">
                    {server.url}
                  </Text>
                )}
                {server.command && (
                  <Card size="small" className={'!mt-2'}>
                    <pre className="text-xs overflow-auto m-0">
                      {server.command}
                      {server.args &&
                        Array.isArray(server.args) &&
                        server.args.length > 0 && (
                          <span> {server.args.join(' ')}</span>
                        )}
                    </pre>
                  </Card>
                )}
              </div>

              {/* Tools Section - Show when expanded and server is active */}
              {showTools && server.is_active && (
                <div className="mt-4">
                  <div className="flex items-center justify-between mb-3">
                    <Text strong className="text-sm flex items-center gap-2">
                      <ToolOutlined />
                      Available Tools
                      {tools.length > 0 && (
                        <Badge
                          count={tools.length}
                          style={{ backgroundColor: '#52c41a' }}
                        />
                      )}
                    </Text>
                    <Button
                      type="text"
                      size="small"
                      icon={<ReloadOutlined />}
                      onClick={e => {
                        e.stopPropagation()
                        handleRefreshTools()
                      }}
                      loading={loadingTools}
                    >
                      Refresh
                    </Button>
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
                                  {tool.usage_count > 0 && (
                                    <Badge
                                      count={tool.usage_count}
                                      style={{
                                        backgroundColor: '#f0f0f0',
                                        color: '#666',
                                      }}
                                    />
                                  )}
                                  <Tooltip title="Auto-approve this tool in all conversations">
                                    <div onClick={e => e.stopPropagation()}>
                                      <Switch
                                        size="small"
                                        checked={
                                          tool.global_auto_approve || false
                                        }
                                        checkedChildren={<CheckOutlined />}
                                        onChange={checked =>
                                          handleToggleAutoApprove(tool, checked)
                                        }
                                      />
                                    </div>
                                  </Tooltip>
                                  <Button
                                    type="link"
                                    size="small"
                                    className="p-0 h-auto text-xs"
                                    onClick={e => {
                                      e.stopPropagation()
                                      handleTestTool(tool)
                                    }}
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

              {/* Show tools message for inactive servers with tools */}
              {showTools &&
                !server.is_active &&
                (server.tool_count ?? 0) > 0 && (
                  <div className="mt-4">
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
        </div>
      </Card>

      {/* Tool Testing Modal */}
      {testingTool && (
        <ToolTestingModal
          tool={testingTool}
          server={server}
          open={!!testingTool}
          onClose={handleCloseTestModal}
        />
      )}
    </>
  )
}
