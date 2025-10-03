import { Button, Form } from 'antd'
import { ToolOutlined } from '@ant-design/icons'
import { useMemo, useState } from 'react'
import { Stores } from '../../../../store'
import { ToolSelectionModal } from './ToolSelectionModal'

interface ToolSelectorProps {
  isDisabled: boolean
}

export const ToolSelector = ({ isDisabled }: ToolSelectorProps) => {
  const form = Form.useFormInstance()
  const selectedTools: Array<{ server_id: string; name: string }> =
    Form.useWatch('enabled_tools', form) || []
  const { servers, tools } = Stores.MCP
  const [isToolModalVisible, setIsToolModalVisible] = useState(false)

  const availableToolsCount = useMemo(() => {
    const enabledActiveServers = servers.filter(
      server => server.enabled && server.is_active,
    )
    return tools.filter(tool =>
      enabledActiveServers.some(server => server.id === tool.server_id),
    ).length
  }, [servers, tools])

  const selectedToolsCount = selectedTools.length

  return (
    <>
      {availableToolsCount > 0 && (
        <Button
          type={selectedToolsCount > 0 ? 'primary' : 'default'}
          disabled={isDisabled}
          title="Select MCP tools"
          onClick={() => setIsToolModalVisible(true)}
        >
          <ToolOutlined />
        </Button>
      )}

      <Form form={form}>
        <ToolSelectionModal
          visible={isToolModalVisible}
          onClose={() => setIsToolModalVisible(false)}
        />
      </Form>
    </>
  )
}
