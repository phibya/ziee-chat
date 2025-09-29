import {
  Button,
  Form,
  Typography,
  Input,
  Select,
  Switch,
  Card,
  Alert,
  Flex,
} from 'antd'
import { InfoCircleOutlined } from '@ant-design/icons'
import { Drawer } from '../../../common/Drawer'
import { useEffect } from 'react'
import {
  closeMCPServerDrawer,
  setMCPServerDrawerLoading,
  Stores,
} from '../../../../store'
import { createMCPServer, updateMCPServer } from '../../../../store/mcp'
import { createSystemServer } from '../../../../store/admin/mcpServers'
import type {
  CreateMCPServerRequest,
  UpdateMCPServerRequest,
  CreateSystemMCPServerRequest,
  UpdateMCPServerRequest as UpdateSystemMCPServerRequest,
} from '../../../../types/api'

const { Text } = Typography
const { TextArea } = Input

const TRANSPORT_TYPES = [
  {
    label: 'Standard I/O',
    value: 'stdio',
    description:
      'Start MCP server as a local process communicating via stdin/stdout',
  },
  {
    label: 'HTTP',
    value: 'http',
    description: 'Connect to MCP server via HTTP/HTTPS endpoint',
  },
  {
    label: 'Server-Sent Events',
    value: 'sse',
    description: 'Connect to MCP server via Server-Sent Events',
  },
]

export function MCPServerDrawer() {
  const [form] = Form.useForm()

  const { open, loading, mode, editingServer } = Stores.UI.MCPServerDrawer
  const { isDesktop } = Stores.Auth

  // Determine if drawer should be open for this mode
  const isOpen =
    open &&
    ['create', 'edit', 'clone', 'create-system', 'edit-system'].includes(mode)

  // Filter transport types based on context
  const getAvailableTransportTypes = () => {
    const isSystemServer = mode === 'create-system' || mode === 'edit-system'

    // Show stdio transport only for system servers or when running as desktop app
    if (isSystemServer || isDesktop) {
      return TRANSPORT_TYPES
    } else {
      // Filter out stdio transport for user servers in web app
      return TRANSPORT_TYPES.filter(type => type.value !== 'stdio')
    }
  }

  const availableTransportTypes = getAvailableTransportTypes()

  // Populate form when editing server changes
  useEffect(() => {
    if (
      editingServer &&
      isOpen &&
      (mode === 'edit' || mode === 'clone' || mode === 'edit-system')
    ) {
      // Check if the existing transport_type is available in current context
      const isTransportAvailable = availableTransportTypes.some(
        type => type.value === editingServer.transport_type,
      )

      const formValues = {
        name:
          mode === 'clone' ? `${editingServer.name}-copy` : editingServer.name,
        display_name:
          mode === 'clone'
            ? `${editingServer.display_name} (Copy)`
            : editingServer.display_name,
        description: editingServer.description,
        transport_type: isTransportAvailable
          ? editingServer.transport_type
          : availableTransportTypes.length > 0
            ? availableTransportTypes[0].value
            : 'http',
        url: editingServer.url,
        command: editingServer.command,
        args: editingServer.args?.join(' ') || '',
        env: editingServer.environment_variables
          ? JSON.stringify(editingServer.environment_variables, null, 2)
          : '',
        enabled: editingServer.enabled,
      }
      form.setFieldsValue(formValues)
    }
  }, [editingServer, isOpen, mode, form, availableTransportTypes, isDesktop])

  const handleSubmit = async () => {
    try {
      const values = await form.validateFields()
      setMCPServerDrawerLoading(true)

      // Parse environment variables from JSON string
      let environmentVariables = {}
      if (values.env && values.env.trim()) {
        try {
          environmentVariables = JSON.parse(values.env)
        } catch (error) {
          console.error('Invalid JSON in environment variables:', error)
          // Use empty object if JSON is invalid
          environmentVariables = {}
        }
      }

      if (mode === 'create' || mode === 'clone') {
        // Create new user server
        const createRequest: CreateMCPServerRequest = {
          name: values.name,
          display_name: values.display_name,
          description: values.description,
          transport_type: values.transport_type,
          url: values.url,
          command: values.command,
          args: values.args ? values.args.split(' ').filter(Boolean) : [],
          environment_variables: environmentVariables,
          enabled: values.enabled ?? true,
        }
        await createMCPServer(createRequest)
      } else if (mode === 'edit') {
        // Update existing user server
        if (!editingServer) return
        const updateRequest: UpdateMCPServerRequest = {
          display_name: values.display_name,
          description: values.description,
          url: values.url,
          command: values.command,
          args: values.args ? values.args.split(' ').filter(Boolean) : [],
          environment_variables: environmentVariables,
          enabled: values.enabled ?? true,
        }
        await updateMCPServer(editingServer.id, updateRequest)
      } else if (mode === 'create-system') {
        // Create new system server
        const createRequest: CreateSystemMCPServerRequest = {
          name: values.name,
          display_name: values.display_name,
          description: values.description,
          transport_type: values.transport_type,
          url: values.url,
          command: values.command,
          args: values.args ? values.args.split(' ').filter(Boolean) : [],
          environment_variables: environmentVariables,
          enabled: values.enabled ?? true,
        }
        await createSystemServer(createRequest)
      } else if (mode === 'edit-system') {
        // Update existing system server
        if (!editingServer) return
        const updateRequest: UpdateSystemMCPServerRequest = {
          display_name: values.display_name,
          description: values.description,
          url: values.url,
          command: values.command,
          args: values.args ? values.args.split(' ').filter(Boolean) : [],
          environment_variables: environmentVariables,
          enabled: values.enabled ?? true,
        }
        await updateMCPServer(editingServer.id, updateRequest)
      }

      closeMCPServerDrawer()
      form.resetFields()
    } catch (error) {
      console.error('Failed to save MCP server:', error)
    } finally {
      setMCPServerDrawerLoading(false)
    }
  }

  const handleClose = () => {
    closeMCPServerDrawer()
    form.resetFields()
  }

  const getTitle = () => {
    switch (mode) {
      case 'create':
        return 'Add MCP Server'
      case 'edit':
        return 'Edit MCP Server'
      case 'clone':
        return 'Clone MCP Server'
      case 'create-system':
        return 'Add System Server'
      case 'edit-system':
        return 'Edit System Server'
      default:
        return 'MCP Server'
    }
  }

  const getButtonText = () => {
    switch (mode) {
      case 'create':
        return 'Create Server'
      case 'edit':
        return 'Update Server'
      case 'clone':
        return 'Clone Server'
      case 'create-system':
        return 'Create System Server'
      case 'edit-system':
        return 'Update System Server'
      default:
        return 'Save'
    }
  }

  const getDescription = () => {
    switch (mode) {
      case 'create':
        return 'Configure a new MCP server to expand your tool capabilities. Choose between different transport types based on your server setup.'
      case 'edit':
        return 'Update the configuration for this MCP server.'
      case 'clone':
        return 'Create a copy of this MCP server with modified settings.'
      case 'create-system':
        return 'Configure a new system-level MCP server. System servers are available to all users and managed by administrators.'
      case 'edit-system':
        return 'Update the configuration for this system-level MCP server. Changes will affect all users who have access to this server.'
      default:
        return ''
    }
  }

  const transportType = Form.useWatch('transport_type', form) || 'stdio'

  const renderTransportFields = () => {
    switch (transportType) {
      case 'http':
      case 'sse':
        return (
          <Form.Item
            name="url"
            label="Server URL"
            rules={[
              { required: true, message: 'Server URL is required' },
              { type: 'url', message: 'Please enter a valid URL' },
            ]}
          >
            <Input
              placeholder={
                transportType === 'http'
                  ? 'https://api.example.com/mcp'
                  : 'https://api.example.com/mcp/events'
              }
            />
          </Form.Item>
        )

      case 'stdio':
        return (
          <>
            <Form.Item
              name="command"
              label="Command"
              rules={[{ required: true, message: 'Command is required' }]}
            >
              <Input placeholder="npx" />
            </Form.Item>
            <Form.Item
              name="args"
              label="Arguments"
              tooltip="Space-separated command arguments"
            >
              <Input placeholder="--verbose --config config.json" />
            </Form.Item>
          </>
        )

      default:
        return null
    }
  }

  return (
    <Drawer
      title={getTitle()}
      open={isOpen}
      onClose={handleClose}
      width={600}
      maskClosable={false}
      footer={[
        <Button key="cancel" onClick={handleClose}>
          Cancel
        </Button>,
        <Button
          key="submit"
          type="primary"
          onClick={handleSubmit}
          loading={loading}
        >
          {getButtonText()}
        </Button>,
      ]}
    >
      <div className="flex flex-col gap-3">
        <Text type="secondary">{getDescription()}</Text>

        <Form
          form={form}
          layout="vertical"
          initialValues={{
            enabled: true,
            transport_type:
              availableTransportTypes.length > 0
                ? availableTransportTypes[0].value
                : 'http',
          }}
          className="flex flex-col gap-3"
        >
          {/* Basic Information */}
          <Card title="Basic Information">
            <Form.Item
              name="name"
              label="Server Name"
              rules={[
                { required: true, message: 'Server name is required' },
                {
                  pattern: /^[a-zA-Z0-9_-]+$/,
                  message:
                    'Name can only contain letters, numbers, underscores, and hyphens',
                },
              ]}
            >
              <Input placeholder="my-mcp-server" />
            </Form.Item>

            <Form.Item
              name="display_name"
              label="Display Name"
              rules={[{ required: true, message: 'Display name is required' }]}
            >
              <Input placeholder="My MCP Server" />
            </Form.Item>

            <Form.Item name="description" label="Description">
              <TextArea
                rows={3}
                placeholder="Brief description of what this MCP server provides"
              />
            </Form.Item>

            <Form.Item name="enabled" label="Enabled" valuePropName="checked">
              <Switch />
            </Form.Item>
          </Card>

          <Flex className="flex-col gap-3">
            {/* Transport Configuration */}
            <Card title="Transport Configuration">
              <Form.Item
                name="transport_type"
                label="Transport Type"
                rules={[
                  { required: true, message: 'Transport type is required' },
                ]}
              >
                <Select
                  optionRender={option => {
                    const type = availableTransportTypes.find(
                      t => t.value === option.value,
                    )
                    return type ? (
                      <div>
                        <div className="font-medium">{type.label}</div>
                        <Text type="secondary" className="text-xs">
                          {type.description}
                        </Text>
                      </div>
                    ) : null
                  }}
                  options={availableTransportTypes.map(type => ({
                    label: type.label,
                    value: type.value,
                  }))}
                />
              </Form.Item>

              {renderTransportFields()}

              {transportType === 'stdio' && (
                <Alert
                  message="Standard I/O Transport"
                  description="The MCP server will be started as a child process. Make sure the command is executable and has the required permissions."
                  type="info"
                  showIcon
                  icon={<InfoCircleOutlined />}
                />
              )}

              {transportType === 'sse' && (
                <Alert
                  message="Server-Sent Events Transport"
                  description="The MCP server will use Server-Sent Events for real-time communication. Ensure the URL supports SSE connections."
                  type="info"
                  showIcon
                />
              )}
            </Card>

            {/* Advanced Settings */}
            <Card title="Advanced Settings">
              <Form.Item
                name="env"
                label="Environment Variables"
                tooltip="JSON object with environment variables for the MCP server"
              >
                <TextArea
                  rows={4}
                  placeholder='{"API_KEY": "your-key", "DEBUG": "true"}'
                />
              </Form.Item>

              <Text type="secondary" className="text-xs">
                For stdio transport, these environment variables will be passed
                to the child process.
              </Text>
            </Card>
          </Flex>
        </Form>
      </div>
    </Drawer>
  )
}
