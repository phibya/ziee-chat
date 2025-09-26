import { useState, useEffect } from 'react'
import {
  App,
  Modal,
  Form,
  Input,
  Button,
  Typography,
  Alert,
  Spin,
  Tag,
  Descriptions,
  Card,
  Select,
  InputNumber,
  Switch,
  Tabs,
} from 'antd'
import {
  PlayCircleOutlined,
  CheckCircleOutlined,
  ExclamationCircleOutlined,
  InfoCircleOutlined,
  CodeOutlined,
  ClockCircleOutlined,
  CopyOutlined,
} from '@ant-design/icons'
import type { MCPServer, MCPToolWithServer } from '../../../../types/api'
import { executeTool } from '../../../../store/mcpExecution'

const { Text } = Typography
const { TextArea } = Input

interface ToolTestingModalProps {
  tool: MCPToolWithServer
  server: MCPServer
  open: boolean
  onClose: () => void
}

interface ToolExecutionResult {
  execution_id: string
  status:
    | 'pending'
    | 'running'
    | 'completed'
    | 'failed'
    | 'cancelled'
    | 'timeout'
  result?: any
  error_message?: string
  duration_ms?: number
}

export function ToolTestingModal({
  tool,
  server,
  open,
  onClose,
}: ToolTestingModalProps) {
  const { message } = App.useApp()
  const [form] = Form.useForm()

  const [executing, setExecuting] = useState(false)
  const [executionResult, setExecutionResult] =
    useState<ToolExecutionResult | null>(null)
  const [parameters, setParameters] = useState<Record<string, any>>({})

  // Parse tool schema to extract parameters
  const toolSchema = tool.input_schema ? JSON.parse(tool.input_schema) : null
  const toolProperties = toolSchema?.properties || {}
  const requiredFields = toolSchema?.required || []

  useEffect(() => {
    // Initialize parameters with default values
    const initialParams: Record<string, any> = {}
    Object.keys(toolProperties).forEach(key => {
      const prop = toolProperties[key]
      if (prop.default !== undefined) {
        initialParams[key] = prop.default
      } else if (prop.type === 'boolean') {
        initialParams[key] = false
      } else if (prop.type === 'number' || prop.type === 'integer') {
        initialParams[key] = 0
      } else {
        initialParams[key] = ''
      }
    })
    setParameters(initialParams)
    form.setFieldsValue(initialParams)
  }, [tool, toolProperties, form])

  const handleExecute = async () => {
    try {
      const values = await form.validateFields()
      setExecuting(true)
      setExecutionResult(null)

      const result = await executeTool(tool.tool_name, values, {
        serverId: server.id,
        conversationId: 'test-execution', // Special test conversation
        autoApprove: true, // Skip approval for testing
        requireApproval: false,
      })

      setExecutionResult({
        execution_id: result.execution_id,
        status: 'completed',
        result: result.result,
        duration_ms: result.duration_ms,
      })

      message.success('Tool executed successfully')
    } catch (error: any) {
      setExecutionResult({
        execution_id: 'error',
        status: 'failed',
        error_message: error.message || 'Unknown error occurred',
      })
      message.error('Tool execution failed')
    } finally {
      setExecuting(false)
    }
  }

  const handleCopyResult = () => {
    if (executionResult?.result) {
      navigator.clipboard.writeText(
        JSON.stringify(executionResult.result, null, 2),
      )
      message.success('Result copied to clipboard')
    }
  }

  const renderParameterInput = (key: string, property: any) => {
    const isRequired = requiredFields.includes(key)

    switch (property.type) {
      case 'boolean':
        return (
          <Form.Item
            key={key}
            name={key}
            label={property.title || key}
            tooltip={property.description}
            valuePropName="checked"
            rules={isRequired ? [{ required: true }] : []}
          >
            <Switch />
          </Form.Item>
        )

      case 'number':
      case 'integer':
        return (
          <Form.Item
            key={key}
            name={key}
            label={property.title || key}
            tooltip={property.description}
            rules={
              isRequired
                ? [{ required: true, message: `${key} is required` }]
                : []
            }
          >
            <InputNumber
              placeholder={property.description}
              min={property.minimum}
              max={property.maximum}
              step={property.type === 'integer' ? 1 : 0.1}
              className="w-full"
            />
          </Form.Item>
        )

      case 'array':
        if (property.items?.enum) {
          return (
            <Form.Item
              key={key}
              name={key}
              label={property.title || key}
              tooltip={property.description}
              rules={
                isRequired
                  ? [{ required: true, message: `${key} is required` }]
                  : []
              }
            >
              <Select
                mode="multiple"
                placeholder={property.description}
                options={property.items.enum.map((option: any) => ({
                  label: option,
                  value: option,
                }))}
              />
            </Form.Item>
          )
        }
        return (
          <Form.Item
            key={key}
            name={key}
            label={property.title || key}
            tooltip={property.description}
            rules={
              isRequired
                ? [{ required: true, message: `${key} is required` }]
                : []
            }
          >
            <Select
              mode="tags"
              placeholder={
                property.description || 'Enter values and press Enter'
              }
              className="w-full"
            />
          </Form.Item>
        )

      default:
        if (property.enum) {
          return (
            <Form.Item
              key={key}
              name={key}
              label={property.title || key}
              tooltip={property.description}
              rules={
                isRequired
                  ? [{ required: true, message: `${key} is required` }]
                  : []
              }
            >
              <Select
                placeholder={property.description}
                options={property.enum.map((option: any) => ({
                  label: option,
                  value: option,
                }))}
              />
            </Form.Item>
          )
        }

        return (
          <Form.Item
            key={key}
            name={key}
            label={property.title || key}
            tooltip={property.description}
            rules={
              isRequired
                ? [{ required: true, message: `${key} is required` }]
                : []
            }
          >
            {property.format === 'textarea' ||
            (property.description && property.description.includes('large')) ? (
              <TextArea
                placeholder={property.description}
                rows={3}
                maxLength={property.maxLength}
              />
            ) : (
              <Input
                placeholder={property.description}
                maxLength={property.maxLength}
              />
            )}
          </Form.Item>
        )
    }
  }

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'completed':
        return <CheckCircleOutlined style={{ color: '#52c41a' }} />
      case 'failed':
        return <ExclamationCircleOutlined style={{ color: '#ff4d4f' }} />
      case 'running':
        return <ClockCircleOutlined style={{ color: '#1890ff' }} />
      default:
        return <InfoCircleOutlined />
    }
  }

  const items = [
    {
      key: 'parameters',
      label: 'Parameters',
      children: (
        <div>
          {Object.keys(toolProperties).length === 0 ? (
            <Alert
              message="No Parameters Required"
              description="This tool doesn't require any parameters."
              type="info"
              showIcon
            />
          ) : (
            <Form form={form} layout="vertical" initialValues={parameters}>
              {Object.entries(toolProperties).map(
                ([key, property]: [string, any]) =>
                  renderParameterInput(key, property),
              )}
            </Form>
          )}
        </div>
      ),
    },
    {
      key: 'info',
      label: 'Tool Info',
      children: (
        <Descriptions column={1} bordered size="small">
          <Descriptions.Item label="Tool Name">
            {tool.tool_name}
          </Descriptions.Item>
          <Descriptions.Item label="Server">
            {server.display_name}
          </Descriptions.Item>
          <Descriptions.Item label="Description">
            {tool.tool_description || 'No description available'}
          </Descriptions.Item>
          <Descriptions.Item label="Usage Count">
            <Tag color="blue">{tool.usage_count} executions</Tag>
          </Descriptions.Item>
          {toolSchema && (
            <Descriptions.Item label="Schema">
              <pre className="text-xs bg-gray-50 p-2 rounded overflow-x-auto">
                {JSON.stringify(toolSchema, null, 2)}
              </pre>
            </Descriptions.Item>
          )}
        </Descriptions>
      ),
    },
  ]

  return (
    <Modal
      title={
        <div className="flex items-center gap-2">
          <CodeOutlined />
          <span>Test Tool: {tool.tool_name}</span>
        </div>
      }
      open={open}
      onCancel={onClose}
      width={800}
      footer={[
        <Button key="cancel" onClick={onClose}>
          Close
        </Button>,
        <Button
          key="execute"
          type="primary"
          icon={executing ? <Spin size="small" /> : <PlayCircleOutlined />}
          loading={executing}
          onClick={handleExecute}
        >
          {executing ? 'Executing...' : 'Execute Tool'}
        </Button>,
      ]}
    >
      <div className="space-y-4">
        <Tabs items={items} />

        {executionResult && (
          <Card
            title={
              <div className="flex items-center gap-2">
                {getStatusIcon(executionResult.status)}
                <span>Execution Result</span>
                {executionResult.duration_ms && (
                  <Tag color="blue">{executionResult.duration_ms}ms</Tag>
                )}
              </div>
            }
            extra={
              executionResult.result && (
                <Button
                  size="small"
                  icon={<CopyOutlined />}
                  onClick={handleCopyResult}
                >
                  Copy
                </Button>
              )
            }
          >
            {executionResult.status === 'failed' ? (
              <Alert
                message="Execution Failed"
                description={executionResult.error_message}
                type="error"
                showIcon
              />
            ) : executionResult.result ? (
              <pre className="bg-gray-50 p-3 rounded overflow-auto max-h-64 text-sm">
                {JSON.stringify(executionResult.result, null, 2)}
              </pre>
            ) : (
              <Text type="secondary">No result data</Text>
            )}
          </Card>
        )}
      </div>
    </Modal>
  )
}
