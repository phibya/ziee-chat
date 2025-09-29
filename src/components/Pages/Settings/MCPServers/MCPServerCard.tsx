import { useState } from 'react'
import { App, Button, Card, Tag, Typography, Tooltip, Switch, Flex } from 'antd'
import {
  EditOutlined,
  ToolOutlined,
  FileTextOutlined,
  PlayCircleOutlined,
  StopOutlined,
} from '@ant-design/icons'
import type { MCPServer } from '../../../../types/api'
import {
  startMCPServer,
  stopMCPServer,
  updateMCPServer,
} from '../../../../store/mcp'
import { openMCPServerDrawer } from '../../../../store/ui/mcpDrawers'
import { MCPToolsPanel } from './MCPToolsPanel'
import { MCPLogsPanel } from './MCPLogsPanel'

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
  const [showLogs, setShowLogs] = useState(false)
  const [operationLoading, setOperationLoading] = useState<string | null>(null)
  const [enableLoading, setEnableLoading] = useState(false)

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

      // Clear tools when server is stopped (but keep logs accessible)
      if (!checked) {
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
    if (newShowTools) {
      setShowLogs(false) // Close logs when opening tools
    }
  }

  const handleToggleLogs = () => {
    const newShowLogs = !showLogs
    setShowLogs(newShowLogs)
    if (newShowLogs) {
      setShowTools(false) // Close tools when opening logs
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
                {isEditable && (
                  <Button
                    type={showLogs ? 'primary' : 'default'}
                    icon={<FileTextOutlined />}
                    onClick={e => {
                      e.stopPropagation()
                      handleToggleLogs()
                    }}
                  >
                    Logs
                  </Button>
                )}
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

              {/* Tools Section */}
              {showTools && (
                <MCPToolsPanel server={server} isEditable={isEditable} />
              )}

              {/* Logs Section */}
              {showLogs && (
                <MCPLogsPanel server={server} isEditable={isEditable} />
              )}
            </div>
          </div>
        </div>
      </Card>
    </>
  )
}
