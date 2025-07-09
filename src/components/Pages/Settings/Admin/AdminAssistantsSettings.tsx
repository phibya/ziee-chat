import React, { useState, useEffect } from 'react'
import {
  Card,
  Table,
  Button,
  Switch,
  Modal,
  Form,
  Input,
  Space,
  Popconfirm,
  Typography,
  Tag,
  Tooltip,
  Select,
  InputNumber,
} from 'antd'
import {
  PlusOutlined,
  EditOutlined,
  DeleteOutlined,
  RobotOutlined,
  CodeOutlined,
  FormOutlined,
} from '@ant-design/icons'
import { ApiClient } from '../../../../api/client'
import { Assistant } from '../../../../types/api/assistant'
import { App } from 'antd'

const { Title, Text } = Typography
const { TextArea } = Input

interface AssistantFormData {
  name: string
  description?: string
  instructions?: string
  parameters?: string
  is_default?: boolean
  is_active?: boolean
}

interface ParameterFormField {
  name: string
  type: 'string' | 'number' | 'boolean'
  value: any
}

export const AdminAssistantsSettings: React.FC = () => {
  const { message } = App.useApp()
  const [assistants, setAssistants] = useState<Assistant[]>([])
  const [loading, setLoading] = useState(false)
  const [modalVisible, setModalVisible] = useState(false)
  const [editingAssistant, setEditingAssistant] = useState<Assistant | null>(
    null,
  )
  const [form] = Form.useForm<AssistantFormData>()
  const [parameterMode, setParameterMode] = useState<'json' | 'form'>('json')
  const [parameterFormFields, setParameterFormFields] = useState<
    ParameterFormField[]
  >([])
  const [parameterJson, setParameterJson] = useState('')

  useEffect(() => {
    fetchAssistants()
  }, [])

  const fetchAssistants = async () => {
    try {
      setLoading(true)
      const response = await ApiClient.Admin.listAssistants({
        page: 1,
        per_page: 100,
      })
      setAssistants(response.assistants)
    } catch (error) {
      message.error('Failed to fetch assistants')
    } finally {
      setLoading(false)
    }
  }

  const handleCreateEdit = async (values: AssistantFormData) => {
    try {
      // Get parameters from current mode
      let parametersString = values.parameters
      if (parameterMode === 'form') {
        parametersString = convertFormToJson(parameterFormFields)
      }

      const requestData = {
        name: values.name,
        description: values.description,
        instructions: values.instructions,
        parameters: parametersString ? JSON.parse(parametersString) : undefined,
        is_template: true, // Always true for admin-created assistants
        is_default: values.is_default,
        is_active: values.is_active,
      }

      if (editingAssistant) {
        await ApiClient.Admin.updateAssistant({
          assistant_id: editingAssistant.id,
          ...requestData,
        })
        message.success('Assistant updated successfully')
      } else {
        await ApiClient.Admin.createAssistant(requestData)
        message.success('Assistant created successfully')
      }

      setModalVisible(false)
      setEditingAssistant(null)
      form.resetFields()
      fetchAssistants()
    } catch (error) {
      message.error('Failed to save assistant')
    }
  }

  const convertJsonToForm = (jsonString: string) => {
    if (!jsonString.trim()) {
      setParameterFormFields([])
      return
    }

    try {
      const parsed = JSON.parse(jsonString)
      const fields: ParameterFormField[] = Object.entries(parsed).map(
        ([key, value]) => ({
          name: key,
          type:
            typeof value === 'number'
              ? 'number'
              : typeof value === 'boolean'
                ? 'boolean'
                : 'string',
          value,
        }),
      )
      setParameterFormFields(fields)
    } catch (error) {
      setParameterFormFields([])
    }
  }

  const convertFormToJson = (fields: ParameterFormField[]) => {
    const obj: any = {}
    fields.forEach(field => {
      if (field.name.trim()) {
        obj[field.name] = field.value
      }
    })
    return JSON.stringify(obj, null, 2)
  }

  const handleParameterModeChange = (mode: 'json' | 'form') => {
    if (mode === 'form' && parameterMode === 'json') {
      // Convert JSON to form
      convertJsonToForm(parameterJson)
    } else if (mode === 'json' && parameterMode === 'form') {
      // Convert form to JSON
      const jsonString = convertFormToJson(parameterFormFields)
      setParameterJson(jsonString)
      form.setFieldsValue({ parameters: jsonString })
    }
    setParameterMode(mode)
  }

  const handleFormFieldChange = (
    index: number,
    field: keyof ParameterFormField,
    value: any,
  ) => {
    const newFields = [...parameterFormFields]
    newFields[index] = { ...newFields[index], [field]: value }
    setParameterFormFields(newFields)
  }

  const addFormField = () => {
    setParameterFormFields([
      ...parameterFormFields,
      { name: '', type: 'string', value: '' },
    ])
  }

  const removeFormField = (index: number) => {
    const newFields = parameterFormFields.filter((_, i) => i !== index)
    setParameterFormFields(newFields)
  }

  const handleDelete = async (assistant: Assistant) => {
    try {
      await ApiClient.Admin.deleteAssistant({ assistant_id: assistant.id })
      message.success('Assistant deleted successfully')
      fetchAssistants()
    } catch (error) {
      message.error('Failed to delete assistant')
    }
  }

  const handleEdit = (assistant: Assistant) => {
    setEditingAssistant(assistant)
    const parametersJson = assistant.parameters
      ? JSON.stringify(assistant.parameters, null, 2)
      : ''
    form.setFieldsValue({
      name: assistant.name,
      description: assistant.description,
      instructions: assistant.instructions,
      parameters: parametersJson,
      is_default: assistant.is_default,
      is_active: assistant.is_active,
    })
    setParameterJson(parametersJson)
    convertJsonToForm(parametersJson)
    setParameterMode('json')
    setModalVisible(true)
  }

  const handleCreate = () => {
    setEditingAssistant(null)
    form.resetFields()
    const defaultParams = JSON.stringify(
      {
        stream: true,
        temperature: 0.7,
        frequency_penalty: 0.7,
        presence_penalty: 0.7,
        top_p: 0.95,
        top_k: 2,
      },
      null,
      2,
    )
    form.setFieldsValue({
      is_default: false,
      is_active: true,
      parameters: defaultParams,
    })
    setParameterJson(defaultParams)
    convertJsonToForm(defaultParams)
    setParameterMode('json')
    setModalVisible(true)
  }

  const columns = [
    {
      title: 'Name',
      dataIndex: 'name',
      key: 'name',
      render: (text: string, record: Assistant) => (
        <Space>
          <RobotOutlined />
          <Text strong>{text}</Text>
          {record.is_default && <Tag color="green">Default</Tag>}
          {!record.is_active && <Tag color="red">Inactive</Tag>}
        </Space>
      ),
    },
    {
      title: 'Description',
      dataIndex: 'description',
      key: 'description',
      render: (text: string) => (
        <Text type="secondary">{text || 'No description'}</Text>
      ),
    },
    {
      title: 'Created By',
      dataIndex: 'created_by',
      key: 'created_by',
      render: (userId: string) => (
        <Text type="secondary">{userId ? 'User' : 'System'}</Text>
      ),
    },
    {
      title: 'Created At',
      dataIndex: 'created_at',
      key: 'created_at',
      render: (date: string) => new Date(date).toLocaleDateString(),
    },
    {
      title: 'Actions',
      key: 'actions',
      render: (_: any, record: Assistant) => (
        <Space>
          <Tooltip title="Edit">
            <Button
              type="text"
              icon={<EditOutlined />}
              onClick={() => handleEdit(record)}
            />
          </Tooltip>
          <Popconfirm
            title="Delete Assistant"
            description="Are you sure you want to delete this assistant?"
            onConfirm={() => handleDelete(record)}
            okText="Yes"
            cancelText="No"
          >
            <Tooltip title="Delete">
              <Button type="text" danger icon={<DeleteOutlined />} />
            </Tooltip>
          </Popconfirm>
        </Space>
      ),
    },
  ]

  return (
    <div>
      <div className="flex justify-between items-center mb-6">
        <div>
          <Title level={3}>Assistants</Title>
          <Text type="secondary">
            Manage template assistants. Default assistants are automatically
            cloned for new users.
          </Text>
        </div>
        <Button type="primary" icon={<PlusOutlined />} onClick={handleCreate}>
          Create Assistant
        </Button>
      </div>

      <Card>
        <Table
          columns={columns}
          dataSource={assistants}
          loading={loading}
          rowKey="id"
          pagination={{ pageSize: 10 }}
        />
      </Card>

      <Modal
        title={editingAssistant ? 'Edit Assistant' : 'Create Assistant'}
        open={modalVisible}
        onCancel={() => {
          setModalVisible(false)
          setEditingAssistant(null)
          form.resetFields()
        }}
        footer={null}
        width={800}
      >
        <Form form={form} onFinish={handleCreateEdit} layout="vertical">
          <Form.Item
            name="name"
            label="Name"
            rules={[{ required: true, message: 'Please enter a name' }]}
          >
            <Input placeholder="Enter assistant name" />
          </Form.Item>

          <Form.Item name="description" label="Description">
            <Input.TextArea
              placeholder="Enter assistant description"
              rows={2}
            />
          </Form.Item>

          <Form.Item name="instructions" label="Instructions">
            <TextArea
              placeholder="Enter assistant instructions (supports markdown)"
              rows={6}
            />
          </Form.Item>

          <Form.Item label="Parameters">
            <div className="mb-3">
              <Space>
                <Button
                  type={parameterMode === 'json' ? 'primary' : 'default'}
                  size="small"
                  icon={<CodeOutlined />}
                  onClick={() => handleParameterModeChange('json')}
                >
                  JSON
                </Button>
                <Button
                  type={parameterMode === 'form' ? 'primary' : 'default'}
                  size="small"
                  icon={<FormOutlined />}
                  onClick={() => handleParameterModeChange('form')}
                >
                  Form
                </Button>
              </Space>
            </div>

            {parameterMode === 'json' ? (
              <Form.Item
                name="parameters"
                rules={[
                  {
                    validator: (_, value) => {
                      if (!value) return Promise.resolve()
                      try {
                        JSON.parse(value)
                        return Promise.resolve()
                      } catch {
                        return Promise.reject('Invalid JSON format')
                      }
                    },
                  },
                ]}
              >
                <TextArea
                  value={parameterJson}
                  onChange={e => {
                    setParameterJson(e.target.value)
                    form.setFieldsValue({ parameters: e.target.value })
                  }}
                  placeholder="Enter parameters as JSON"
                  rows={8}
                  style={{ fontFamily: 'monospace' }}
                />
              </Form.Item>
            ) : (
              <div>
                <div className="space-y-3">
                  {parameterFormFields.map((field, index) => (
                    <div key={index} className="flex gap-2 items-center">
                      <Input
                        placeholder="Field name"
                        value={field.name}
                        onChange={e =>
                          handleFormFieldChange(index, 'name', e.target.value)
                        }
                        style={{ width: 150 }}
                      />
                      <Select
                        value={field.type}
                        onChange={value =>
                          handleFormFieldChange(index, 'type', value)
                        }
                        style={{ width: 100 }}
                      >
                        <Select.Option value="string">String</Select.Option>
                        <Select.Option value="number">Number</Select.Option>
                        <Select.Option value="boolean">Boolean</Select.Option>
                      </Select>
                      {field.type === 'boolean' ? (
                        <Switch
                          checked={field.value}
                          onChange={checked =>
                            handleFormFieldChange(index, 'value', checked)
                          }
                        />
                      ) : field.type === 'number' ? (
                        <InputNumber
                          value={field.value}
                          onChange={value =>
                            handleFormFieldChange(index, 'value', value || 0)
                          }
                          style={{ width: 120 }}
                        />
                      ) : (
                        <Input
                          value={field.value}
                          onChange={e =>
                            handleFormFieldChange(
                              index,
                              'value',
                              e.target.value,
                            )
                          }
                          style={{ width: 120 }}
                        />
                      )}
                      <Button
                        type="text"
                        danger
                        onClick={() => removeFormField(index)}
                        icon={<DeleteOutlined />}
                      />
                    </div>
                  ))}
                </div>
                <Button
                  type="dashed"
                  onClick={addFormField}
                  className="mt-3"
                  icon={<PlusOutlined />}
                >
                  Add Field
                </Button>
              </div>
            )}
          </Form.Item>

          <Form.Item name="is_default" label="Default" valuePropName="checked">
            <Switch />
          </Form.Item>

          <Form.Item name="is_active" label="Active" valuePropName="checked">
            <Switch />
          </Form.Item>

          <Form.Item>
            <Space>
              <Button type="primary" htmlType="submit">
                {editingAssistant ? 'Update' : 'Create'}
              </Button>
              <Button
                onClick={() => {
                  setModalVisible(false)
                  setEditingAssistant(null)
                  form.resetFields()
                }}
              >
                Cancel
              </Button>
            </Space>
          </Form.Item>
        </Form>
      </Modal>
    </div>
  )
}
