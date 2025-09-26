import { Form, Input, Select, Switch, Typography, Card, Alert } from 'antd'
import { InfoCircleOutlined } from '@ant-design/icons'
import type { FormInstance } from 'antd'
import type { MCPServer } from '../../../../../types/api'

const { Text } = Typography
const { TextArea } = Input

interface MCPServerConfigFormProps {
  form: FormInstance
  mode: 'create' | 'edit'
  initialServer?: MCPServer | null
}

const TRANSPORT_TYPES = [
  {
    label: 'HTTP/HTTPS',
    value: 'http',
    description: 'Connect to MCP server via HTTP/HTTPS endpoint',
  },
  {
    label: 'WebSocket',
    value: 'websocket',
    description: 'Connect to MCP server via WebSocket',
  },
  {
    label: 'Process/Command',
    value: 'process',
    description: 'Start MCP server as a local process',
  },
  {
    label: 'SSH',
    value: 'ssh',
    description: 'Connect to MCP server via SSH tunnel',
  },
]

export function MCPServerConfigForm({ form }: MCPServerConfigFormProps) {
  const transportType = Form.useWatch('transport_type', form) || 'http'

  const renderTransportFields = () => {
    switch (transportType) {
      case 'http':
      case 'websocket':
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
                  : 'ws://localhost:8080/mcp'
              }
            />
          </Form.Item>
        )

      case 'process':
        return (
          <>
            <Form.Item
              name="command"
              label="Command"
              rules={[{ required: true, message: 'Command is required' }]}
            >
              <Input placeholder="python /path/to/mcp-server.py" />
            </Form.Item>
            <Form.Item
              name="args"
              label="Arguments"
              tooltip="Space-separated command arguments"
            >
              <Input placeholder="--port 8080 --verbose" />
            </Form.Item>
          </>
        )

      case 'ssh':
        return (
          <>
            <Form.Item
              name="url"
              label="SSH Connection"
              rules={[
                {
                  required: true,
                  message: 'SSH connection string is required',
                },
              ]}
            >
              <Input placeholder="user@hostname:port" />
            </Form.Item>
            <Form.Item
              name="command"
              label="Remote Command"
              rules={[
                { required: true, message: 'Remote command is required' },
              ]}
            >
              <Input placeholder="python /remote/path/to/mcp-server.py" />
            </Form.Item>
          </>
        )

      default:
        return null
    }
  }

  return (
    <Form
      form={form}
      layout="vertical"
      initialValues={{
        enabled: true,
        transport_type: 'http',
      }}
    >
      {/* Basic Information */}
      <Card title="Basic Information" className="mb-4">
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

      {/* Transport Configuration */}
      <Card title="Transport Configuration" className="mb-4">
        <Form.Item
          name="transport_type"
          label="Transport Type"
          rules={[{ required: true, message: 'Transport type is required' }]}
        >
          <Select
            options={TRANSPORT_TYPES.map(type => ({
              label: (
                <div>
                  <div className="font-medium">{type.label}</div>
                  <Text type="secondary" className="text-xs">
                    {type.description}
                  </Text>
                </div>
              ),
              value: type.value,
            }))}
          />
        </Form.Item>

        {renderTransportFields()}

        {transportType === 'process' && (
          <Alert
            message="Process Transport"
            description="The MCP server will be started as a child process. Make sure the command is accessible and has the required permissions."
            type="info"
            showIcon
            icon={<InfoCircleOutlined />}
            className="mb-4"
          />
        )}

        {transportType === 'ssh' && (
          <Alert
            message="SSH Transport"
            description="Make sure SSH key authentication is set up for the target host. Password authentication is not supported."
            type="warning"
            showIcon
            className="mb-4"
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
          For process transport, these environment variables will be passed to
          the child process.
        </Text>
      </Card>
    </Form>
  )
}
