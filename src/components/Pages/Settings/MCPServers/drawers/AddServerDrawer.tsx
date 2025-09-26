import { Button, Form, Typography } from 'antd'
import { Drawer } from '../../../../common/Drawer'
import {
  closeMCPServerDrawer,
  setMCPServerDrawerLoading,
  Stores,
} from '../../../../../store'
import { createMCPServer } from '../../../../../store/mcp'
import type { CreateMCPServerRequest } from '../../../../../types/api'
import { MCPServerConfigForm } from '../common/MCPServerConfigForm'

const { Text } = Typography

export function AddServerDrawer() {
  const [form] = Form.useForm()

  const { open, loading, mode } = Stores.UI.MCPServerDrawer

  const isAddMode = open && mode === 'create'

  const handleSubmit = async () => {
    try {
      const values = await form.validateFields()
      setMCPServerDrawerLoading(true)

      // Transform form values to API format
      const createRequest: CreateMCPServerRequest = {
        name: values.name,
        display_name: values.display_name,
        description: values.description,
        transport_type: values.transport_type,
        url: values.url,
        command: values.command,
        args: values.args ? values.args.split(' ').filter(Boolean) : [],
        environment_variables: values.env || {},
        enabled: values.enabled ?? true,
      }

      await createMCPServer(createRequest)
      closeMCPServerDrawer()
      form.resetFields()
    } catch (error) {
      console.error('Failed to create MCP server:', error)
    } finally {
      setMCPServerDrawerLoading(false)
    }
  }

  const handleClose = () => {
    closeMCPServerDrawer()
    form.resetFields()
  }

  return (
    <Drawer
      title="Add MCP Server"
      open={isAddMode}
      onClose={handleClose}
      width={600}
      footer={
        <div className="flex justify-between">
          <Button onClick={handleClose}>Cancel</Button>
          <Button type="primary" onClick={handleSubmit} loading={loading}>
            Create Server
          </Button>
        </div>
      }
    >
      <div className="space-y-4">
        <Text type="secondary">
          Configure a new MCP server to expand your tool capabilities. Choose
          between different transport types based on your server setup.
        </Text>

        <MCPServerConfigForm form={form} mode="create" />
      </div>
    </Drawer>
  )
}
