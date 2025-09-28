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
import { DivScrollY } from '../../../common/DivScrollY'
import {
  EditOutlined,
  ToolOutlined,
  ReloadOutlined,
  PlayCircleOutlined,
  StopOutlined,
  CheckOutlined,
  CloseOutlined,
} from '@ant-design/icons'
import type { MCPServer, MCPToolWithApproval } from '../../../../types/api'
import {
  startMCPServer,
  stopMCPServer,
  getServerTools,
  setToolGlobalApproval,
  removeToolGlobalApproval,
  updateMCPServer,
} from '../../../../store/mcp'
import { openMCPServerDrawer } from '../../../../store/ui/mcpDrawers'

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
  const [tools, setTools] = useState<MCPToolWithApproval[]>([])
  const [loadingTools, setLoadingTools] = useState(false)
  const [operationLoading, setOperationLoading] = useState<string | null>(null)
  const [enableLoading, setEnableLoading] = useState(false)
  const [bulkApprovalLoading, setBulkApprovalLoading] = useState(false)

  // Load tools when server is active and tools section is expanded
  const loadServerTools = async () => {
    if (!server.is_active || loadingTools) return

    setLoadingTools(true)
    try {
      const serverTools = await getServerTools(server.id)
      // The API now returns MCPToolWithApproval[] with approval status
      setTools(serverTools)
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

  const handleToggleAutoApprove = async (
    tool: MCPToolWithApproval,
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
            ? {
                ...t,
                is_auto_approved: autoApprove,
                approval_source: autoApprove ? 'global' : undefined,
              }
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

  const handleBulkToggleAutoApprove = async (autoApprove: boolean) => {
    if (tools.length === 0) return

    setBulkApprovalLoading(true)
    const action = autoApprove ? 'enable' : 'disable'

    try {
      const promises = tools.map(tool => {
        if (autoApprove) {
          return setToolGlobalApproval(tool.server_id, tool.tool_name, {
            auto_approve: true,
          })
        } else {
          return removeToolGlobalApproval(tool.server_id, tool.tool_name)
        }
      })

      await Promise.all(promises)

      // Update all tools in local state
      setTools(prevTools =>
        prevTools.map(t => ({
          ...t,
          is_auto_approved: autoApprove,
          approval_source: autoApprove ? 'global' : undefined,
        })),
      )

      message.success(`Auto-approve ${action}d for all ${tools.length} tools`)
    } catch (error) {
      console.error(`Failed to ${action} bulk auto-approve:`, error)
      message.error(`Failed to ${action} auto-approve for some tools`)
    } finally {
      setBulkApprovalLoading(false)
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
                      <span>Available Tools</span>
                      {tools.length > 0 && (
                        <span className="ml-2">
                          <Badge
                            count={tools.length}
                            style={{ backgroundColor: '#52c41a' }}
                          />
                        </span>
                      )}
                    </Text>
                    <div className="flex items-center gap-1">
                      {tools.length > 0 &&
                        (!server.is_system || !isEditable) && (
                          <>
                            <Tooltip title="Enable auto-approve for all tools">
                              <Button
                                type="text"
                                size="small"
                                icon={<CheckOutlined />}
                                onClick={e => {
                                  e.stopPropagation()
                                  handleBulkToggleAutoApprove(true)
                                }}
                                loading={bulkApprovalLoading}
                                disabled={tools.every(t => t.is_auto_approved)}
                              >
                                Approve All
                              </Button>
                            </Tooltip>
                            <Tooltip title="Disable auto-approve for all tools">
                              <Button
                                type="text"
                                size="small"
                                icon={<CloseOutlined />}
                                onClick={e => {
                                  e.stopPropagation()
                                  handleBulkToggleAutoApprove(false)
                                }}
                                loading={bulkApprovalLoading}
                                disabled={tools.every(t => !t.is_auto_approved)}
                              >
                                Reject All
                              </Button>
                            </Tooltip>
                          </>
                        )}
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
                    <DivScrollY
                      style={{ maxHeight: '500px' }}
                      className="w-full"
                    >
                      <div className="w-full">
                        <List
                          size="small"
                          dataSource={tools}
                          renderItem={tool => (
                            <List.Item className="px-0 py-1">
                              <div className="w-full">
                                <div className="flex items-center justify-between mb-2">
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
                                    {(!server.is_system || !isEditable) && (
                                      <div className="flex items-center gap-2">
                                        <Text className="text-xs">
                                          Auto approve
                                        </Text>
                                        <Tooltip
                                          title={`Auto-approve this tool in all conversations${tool.is_auto_approved && tool.approval_source === 'global' ? ' (globally approved)' : ''}`}
                                        >
                                          <Switch
                                            size="small"
                                            checked={
                                              tool.is_auto_approved || false
                                            }
                                            onChange={checked =>
                                              handleToggleAutoApprove(
                                                tool,
                                                checked,
                                              )
                                            }
                                          />
                                        </Tooltip>
                                        {tool.is_auto_approved &&
                                          tool.approval_expires_at && (
                                            <Tooltip
                                              title={`Approval expires at ${new Date(tool.approval_expires_at).toLocaleString()}`}
                                            >
                                              <Tag
                                                color="orange"
                                                className="text-xs"
                                              >
                                                Expires
                                              </Tag>
                                            </Tooltip>
                                          )}
                                      </div>
                                    )}
                                  </div>
                                </div>
                                {tool.tool_description && (
                                  <Text
                                    type="secondary"
                                    className="text-xs block"
                                  >
                                    {tool.tool_description}
                                  </Text>
                                )}
                              </div>
                            </List.Item>
                          )}
                        />
                      </div>
                    </DivScrollY>
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
    </>
  )
}
