import { Collapse, Flex, Form, Modal, Switch, Tag, Typography } from 'antd'
import { useEffect, useMemo } from 'react'
import { DivScrollY } from '../../../common/DivScrollY'
import { Drawer } from '../../../common/Drawer'
import type { MCPTool } from '../../../../types'
import { Stores } from '../../../../store'
import { useWindowMinSize } from '../../../hooks/useWindowMinSize.ts'

const { Text, Paragraph } = Typography

interface ToolSelectionModalProps {
  visible: boolean
  onClose: () => void
}

export const ToolSelectionModal = ({
  visible,
  onClose,
}: ToolSelectionModalProps) => {
  const form = Form.useFormInstance()
  const selectedTools: Array<{ server_id: string; name: string }> =
    Form.useWatch('enabled_tools', form) || []
  const selectedRags: string[] = Form.useWatch('enabled_rag_ids', form) || []
  const { servers, tools } = Stores.MCP
  const { ragInstances } = Stores.RAG
  const windowMinSize = useWindowMinSize()

  const availableTools = useMemo(() => {
    const enabledActiveServers = servers.filter(
      server => server.enabled && server.is_active,
    )
    return tools.filter(tool =>
      enabledActiveServers.some(server => server.id === tool.server_id),
    )
  }, [servers, tools])

  const availableRags = useMemo(() => {
    return ragInstances.filter(rag => rag.enabled && rag.is_active)
  }, [ragInstances])

  // Initialize with all tools and RAGs enabled by default
  useEffect(() => {
    const currentTools = form.getFieldValue('enabled_tools')
    if (
      availableTools.length > 0 &&
      (!currentTools || currentTools.length === 0)
    ) {
      form.setFieldValue(
        'enabled_tools',
        availableTools.map(tool => ({
          server_id: tool.server_id,
          name: tool.tool_name,
        })),
      )
    }

    const currentRags = form.getFieldValue('enabled_rag_ids')
    if (
      availableRags.length > 0 &&
      (!currentRags || currentRags.length === 0)
    ) {
      form.setFieldValue(
        'enabled_rag_ids',
        availableRags.map(rag => rag.id),
      )
    }
  }, [availableTools, availableRags, form])

  // Group tools by server
  const toolsByServer = new Map<string, MCPTool[]>()
  availableTools.forEach(tool => {
    const serverTools = toolsByServer.get(tool.server_id) || []
    serverTools.push(tool)
    toolsByServer.set(tool.server_id, serverTools)
  })

  const handleServerToggle = (serverId: string, checked: boolean) => {
    const serverTools = toolsByServer.get(serverId) || []
    if (checked) {
      const serverToolsToAdd = serverTools
        .filter(
          tool =>
            !selectedTools.some(
              t => t.server_id === serverId && t.name === tool.tool_name,
            ),
        )
        .map(tool => ({
          server_id: serverId,
          name: tool.tool_name,
        }))
      form.setFieldValue('enabled_tools', [
        ...selectedTools,
        ...serverToolsToAdd,
      ])
    } else {
      form.setFieldValue(
        'enabled_tools',
        selectedTools.filter(t => t.server_id !== serverId),
      )
    }
  }

  const handleToolToggle = (
    serverId: string,
    toolName: string,
    checked: boolean,
  ) => {
    if (checked) {
      form.setFieldValue('enabled_tools', [
        ...selectedTools,
        { server_id: serverId, name: toolName },
      ])
    } else {
      form.setFieldValue(
        'enabled_tools',
        selectedTools.filter(
          t => !(t.server_id === serverId && t.name === toolName),
        ),
      )
    }
  }

  const handleRagToggle = (ragId: string, checked: boolean) => {
    if (checked) {
      form.setFieldValue('enabled_rag_ids', [...selectedRags, ragId])
    } else {
      form.setFieldValue(
        'enabled_rag_ids',
        selectedRags.filter(id => id !== ragId),
      )
    }
  }

  const allRagsSelected = availableRags.every(rag =>
    selectedRags.includes(rag.id),
  )

  const handleAllRagsToggle = (checked: boolean) => {
    if (checked) {
      form.setFieldValue(
        'enabled_rag_ids',
        availableRags.map(rag => rag.id),
      )
    } else {
      form.setFieldValue('enabled_rag_ids', [])
    }
  }

  const content = (
    <div
      className={'w-full flex-col px-3 pb-1'}
      style={{
        paddingLeft: windowMinSize.xs ? 0 : undefined,
        paddingRight: windowMinSize.xs ? 0 : undefined,
      }}
    >
      {availableRags.length > 0 && (
        <Collapse
          className={'w-full !mb-3 !bg-transparent'}
          items={[
            {
              key: 'rags',
              label: (
                <Flex justify="space-between" align="center" className="w-full">
                  <Flex gap={8} align="center">
                    <Switch
                      size="small"
                      checked={allRagsSelected}
                      onClick={(_, e) => e.stopPropagation()}
                      onChange={handleAllRagsToggle}
                    />
                    <Text strong>RAG</Text>
                  </Flex>
                  <Text type="secondary" className="text-xs">
                    {availableRags.length} instance
                    {availableRags.length !== 1 ? 's' : ''}
                  </Text>
                </Flex>
              ),
              children: (
                <div className="flex flex-col gap-2">
                  {availableRags.map(rag => {
                    const isSelected = selectedRags.includes(rag.id)
                    const hasDescription =
                      rag.description && rag.description.length > 0

                    return (
                      <div key={rag.id} className="flex flex-col gap-1">
                        <div className="flex items-center gap-2">
                          <Switch
                            size="small"
                            checked={isSelected}
                            onChange={checked =>
                              handleRagToggle(rag.id, checked)
                            }
                          />
                          <Flex gap={4} align="center">
                            <Text>{rag.display_name}</Text>
                            {rag.is_system && (
                              <Tag color="blue" className="text-xs">
                                System
                              </Tag>
                            )}
                          </Flex>
                        </div>
                        {hasDescription && (
                          <Paragraph
                            type="secondary"
                            className="text-xs !mb-0"
                            ellipsis={{
                              rows: 1,
                              expandable: 'collapsible',
                            }}
                          >
                            {rag.description}
                          </Paragraph>
                        )}
                      </div>
                    )
                  })}
                </div>
              ),
            },
          ]}
        />
      )}

      {/* MCP Tools Section */}
      <Collapse
        className={'w-full !bg-transparent'}
        items={Array.from(toolsByServer.entries())
          .map(([serverId, serverTools]) => {
            const server = servers.find(s => s.id === serverId)
            if (!server) return null

            const allServerToolsSelected = serverTools.every(tool =>
              selectedTools.some(
                t => t.server_id === serverId && t.name === tool.tool_name,
              ),
            )

            return {
              key: serverId,
              label: (
                <Flex justify="space-between" align="center" className="w-full">
                  <Flex gap={8} align="center">
                    <Switch
                      size="small"
                      checked={allServerToolsSelected}
                      onClick={(_, e) => e.stopPropagation()}
                      onChange={checked =>
                        handleServerToggle(serverId, checked)
                      }
                    />
                    <Text strong>{server.display_name}</Text>
                    <Tag color={server.is_system ? 'blue' : 'default'}>
                      {server.is_system ? 'System' : 'User'}
                    </Tag>
                  </Flex>
                  <Text type="secondary" className="text-xs">
                    {serverTools.length} tool
                    {serverTools.length !== 1 ? 's' : ''}
                  </Text>
                </Flex>
              ),
              children: (
                <div className="flex flex-col gap-2">
                  {serverTools.map(tool => {
                    const isSelected = selectedTools.some(
                      t =>
                        t.server_id === serverId && t.name === tool.tool_name,
                    )
                    const toolKey = `${serverId}-${tool.tool_name}`
                    const hasDescription =
                      tool.tool_description && tool.tool_description.length > 0

                    return (
                      <div key={toolKey} className="flex flex-col gap-1">
                        <div className="flex items-center gap-2">
                          <Switch
                            size="small"
                            checked={isSelected}
                            onChange={checked =>
                              handleToolToggle(
                                serverId,
                                tool.tool_name,
                                checked,
                              )
                            }
                          />
                          <Text>{tool.tool_name}</Text>
                        </div>
                        {hasDescription && (
                          <Paragraph
                            type="secondary"
                            className="text-xs !mb-0"
                            ellipsis={{
                              rows: 1,
                              expandable: 'collapsible',
                            }}
                          >
                            {tool.tool_description}
                          </Paragraph>
                        )}
                      </div>
                    )
                  })}
                </div>
              ),
            }
          })
          .filter(
            (
              item,
            ): item is {
              key: string
              label: JSX.Element
              children: JSX.Element
            } => item !== null,
          )}
      />

      {availableTools.length === 0 && (
        <Text type="secondary" className="text-center py-4 block">
          No tools available from enabled and active MCP servers
        </Text>
      )}
    </div>
  )

  if (windowMinSize.xs) {
    return (
      <Drawer title="RAG & Tools" open={visible} onClose={onClose}>
        {content}
      </Drawer>
    )
  }

  return (
    <Modal
      title="RAG & Tools"
      open={visible}
      onOk={onClose}
      onCancel={onClose}
      centered
      width={700}
      classNames={{
        content: '!px-0',
        header: '!px-3 !pb-1',
        footer: '!px-3',
        body: 'flex max-h-[80vh] overflow-hidden',
      }}
    >
      <DivScrollY className="w-full">
        <div className="flex w-full">{content}</div>
      </DivScrollY>
    </Modal>
  )
}
