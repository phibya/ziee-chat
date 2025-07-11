import React, { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  App,
  Button,
  Card,
  Form,
  Input,
  InputNumber,
  Modal,
  Popconfirm,
  Select,
  Space,
  Switch,
  Table,
  Tag,
  Tooltip,
  Typography,
} from 'antd'
import {
  CodeOutlined,
  DeleteOutlined,
  EditOutlined,
  FormOutlined,
  PlusOutlined,
  RobotOutlined,
} from '@ant-design/icons'
import { useShallow } from 'zustand/react/shallow'
import { Assistant } from '../../../../types/api/assistant'
import { PageContainer } from '../../../common/PageContainer'
import { useAdminStore } from '../../../../store/admin'

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
  const { t } = useTranslation()
  const { message } = App.useApp()

  // Admin store
  const {
    assistants,
    loading,
    error,
    loadAssistants,
    createAssistant,
    updateAssistant,
    deleteAssistant,
    clearError,
  } = useAdminStore(
    useShallow(state => ({
      assistants: state.assistants,
      loading: state.loading,
      creating: state.creating,
      updating: state.updating,
      deleting: state.deleting,
      error: state.error,
      loadAssistants: state.loadAssistants,
      createAssistant: state.createAssistant,
      updateAssistant: state.updateAssistant,
      deleteAssistant: state.deleteAssistant,
      clearError: state.clearError,
    })),
  )

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
    loadAssistants()
  }, [loadAssistants])

  // Show errors
  useEffect(() => {
    if (error) {
      message.error(error)
      clearError()
    }
  }, [error, message, clearError])

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
        await updateAssistant(editingAssistant.id, requestData)
        message.success('Assistant updated successfully')
      } else {
        await createAssistant(requestData)
        message.success('Assistant created successfully')
      }

      setModalVisible(false)
      setEditingAssistant(null)
      form.resetFields()
    } catch (error) {
      console.error('Failed to save assistant:', error)
      // Error is handled by the store
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
    } catch {
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
      await deleteAssistant(assistant.id)
      message.success('Assistant deleted successfully')
    } catch (error) {
      console.error('Failed to delete assistant:', error)
      // Error is handled by the store
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
      title: t('labels.name'),
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
      title: t('labels.description'),
      dataIndex: 'description',
      key: 'description',
      render: (text: string) => (
        <Text type="secondary">{text || 'No description'}</Text>
      ),
    },
    {
      title: t('admin.assistants.createdBy'),
      dataIndex: 'created_by',
      key: 'created_by',
      render: (userId: string) => (
        <Text type="secondary">{userId ? 'User' : 'System'}</Text>
      ),
    },
    {
      title: t('labels.created'),
      dataIndex: 'created_at',
      key: 'created_at',
      render: (date: string) => new Date(date).toLocaleDateString(),
    },
    {
      title: t('labels.actions'),
      key: 'actions',
      render: (_: any, record: Assistant) => (
        <Space>
          <Tooltip title={t('buttons.edit')}>
            <Button
              type="text"
              icon={<EditOutlined />}
              onClick={() => handleEdit(record)}
            />
          </Tooltip>
          <Popconfirm
            title={t('assistants.deleteAssistant')}
            description={t('assistants.deleteConfirm')}
            onConfirm={() => handleDelete(record)}
            okText="Yes"
            cancelText="No"
          >
            <Tooltip title={t('buttons.delete')}>
              <Button type="text" danger icon={<DeleteOutlined />} />
            </Tooltip>
          </Popconfirm>
        </Space>
      ),
    },
  ]

  return (
    <PageContainer>
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
          maskClosable={false}
        >
          <Form form={form} onFinish={handleCreateEdit} layout="vertical">
            <Form.Item
              name="name"
              label={t('labels.name')}
              rules={[{ required: true, message: 'Please enter a name' }]}
            >
              <Input placeholder={t('forms.enterAssistantName')} />
            </Form.Item>

            <Form.Item name="description" label={t('labels.description')}>
              <Input.TextArea
                placeholder={t('forms.enterAssistantDescription')}
                rows={2}
              />
            </Form.Item>

            <Form.Item name="instructions" label={t('labels.instructions')}>
              <TextArea
                placeholder={t('forms.enterAssistantInstructions')}
                rows={6}
              />
            </Form.Item>

            <Form.Item label={t('labels.parameters')}>
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
                    placeholder={t('forms.enterParametersJson')}
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

            <Form.Item
              name="is_default"
              label="Default"
              valuePropName="checked"
            >
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
    </PageContainer>
  )
}
