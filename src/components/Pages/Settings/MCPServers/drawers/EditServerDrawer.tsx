import { Button, Form, Typography } from 'antd'
import { Drawer } from '../../../../common/Drawer'
import { useEffect } from 'react'
import {
  closeMCPServerDrawer,
  setMCPServerDrawerLoading,
  Stores,
} from '../../../../../store'
import { updateMCPServer } from '../../../../../store/mcp'
import type { UpdateMCPServerRequest } from '../../../../../types/api'
import { MCPServerConfigForm } from '../common/MCPServerConfigForm'

const { Text } = Typography

export function EditServerDrawer() {
  const [form] = Form.useForm()

  const { open, loading, mode, editingServer } = Stores.UI.MCPServerDrawer

  const isEditMode = open && (mode === 'edit' || mode === 'clone')

  // Populate form when editing server changes
  useEffect(() => {
    if (editingServer && isEditMode) {
      const formValues = {
        name:
          mode === 'clone' ? `${editingServer.name}-copy` : editingServer.name,
        display_name:
          mode === 'clone'
            ? `${editingServer.display_name} (Copy)`
            : editingServer.display_name,
        description: editingServer.description,
        transport_type: editingServer.transport_type,
        url: editingServer.url,
        command: editingServer.command,
        args: editingServer.args?.join(' ') || '',
        env: editingServer.environment_variables || {},
        enabled: editingServer.enabled,
      }
      form.setFieldsValue(formValues)
    }
  }, [editingServer, isEditMode, mode, form])

  const handleSubmit = async () => {
    if (!editingServer) return

    try {
      const values = await form.validateFields()
      setMCPServerDrawerLoading(true)

      // Transform form values to API format
      const updateRequest: UpdateMCPServerRequest = {
        display_name: values.display_name,
        description: values.description,
        url: values.url,
        command: values.command,
        args: values.args ? values.args.split(' ').filter(Boolean) : [],
        environment_variables: values.env || {},
        enabled: values.enabled ?? true,
      }

      if (mode === 'clone') {
        // For clone mode, we need to create a new server, but this is handled by AddServerDrawer
        // This shouldn't happen as clone uses create mode
        console.warn(
          'Clone mode in EditServerDrawer - this should use create mode',
        )
      } else {
        await updateMCPServer(editingServer.id, updateRequest)
      }

      closeMCPServerDrawer()
      form.resetFields()
    } catch (error) {
      console.error('Failed to update MCP server:', error)
    } finally {
      setMCPServerDrawerLoading(false)
    }
  }

  const handleClose = () => {
    closeMCPServerDrawer()
    form.resetFields()
  }

  const getTitle = () => {
    if (mode === 'clone') return 'Clone MCP Server'
    return 'Edit MCP Server'
  }

  const getButtonText = () => {
    if (mode === 'clone') return 'Clone Server'
    return 'Update Server'
  }

  return (
    <Drawer
      title={getTitle()}
      open={isEditMode}
      onClose={handleClose}
      width={600}
      footer={
        <div className="flex justify-between">
          <Button onClick={handleClose}>Cancel</Button>
          <Button type="primary" onClick={handleSubmit} loading={loading}>
            {getButtonText()}
          </Button>
        </div>
      }
    >
      <div className="space-y-4">
        <Text type="secondary">
          {mode === 'clone'
            ? 'Create a copy of this MCP server with modified settings.'
            : 'Update the configuration for this MCP server.'}
        </Text>

        <MCPServerConfigForm
          form={form}
          mode={mode === 'clone' ? 'create' : 'edit'}
          initialServer={editingServer}
        />
      </div>
    </Drawer>
  )
}
