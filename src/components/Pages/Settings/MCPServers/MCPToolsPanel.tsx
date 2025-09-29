import { useState, useEffect } from 'react'
import {
  App,
  Button,
  Tooltip,
  Switch,
  List,
  Empty,
  Badge,
  Typography,
} from 'antd'
import { DivScrollY } from '../../../common/DivScrollY'
import {
  ToolOutlined,
  ReloadOutlined,
  CheckOutlined,
  CloseOutlined,
} from '@ant-design/icons'
import type { MCPServer, MCPToolWithApproval } from '../../../../types/api'
import {
  getServerTools,
  setToolGlobalApproval,
  removeToolGlobalApproval,
} from '../../../../store/mcp'

const { Text } = Typography

interface MCPToolsPanelProps {
  server: MCPServer
  isEditable?: boolean
}

export function MCPToolsPanel({
  server,
  isEditable = true,
}: MCPToolsPanelProps) {
  const { message } = App.useApp()
  const [tools, setTools] = useState<MCPToolWithApproval[]>([])
  const [loadingTools, setLoadingTools] = useState(false)
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

  // Auto-load tools when component mounts for an active server
  useEffect(() => {
    if (server.is_active && tools.length === 0) {
      loadServerTools()
    }
  }, [server.is_active])

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
                                  <Badge
                                    status="warning"
                                    text="Expires"
                                  />
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
  )
}