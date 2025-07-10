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
  Row,
  Col,
  Avatar,
  Select,
  InputNumber,
} from 'antd'
import {
  PlusOutlined,
  EditOutlined,
  DeleteOutlined,
  RobotOutlined,
  CopyOutlined,
  CodeOutlined,
  FormOutlined,
} from '@ant-design/icons'
import { ApiClient } from '../../api/client'
import { Assistant } from '../../types/api/assistant'
import { App } from 'antd'
import { useTranslation } from 'react-i18next'
import { PageContainer } from '../common/PageContainer'

const { Title, Text } = Typography
const { TextArea } = Input

interface AssistantFormData {
  name: string
  description?: string
  instructions?: string
  parameters?: string
  is_active?: boolean
}

interface ParameterFormField {
  name: string
  type: 'string' | 'number' | 'boolean'
  value: any
}

export const AssistantsPage: React.FC = () => {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const [assistants, setAssistants] = useState<Assistant[]>([])
  const [templateAssistants, setTemplateAssistants] = useState<Assistant[]>([])
  const [loading, setLoading] = useState(false)
  const [modalVisible, setModalVisible] = useState(false)
  const [templateModalVisible, setTemplateModalVisible] = useState(false)
  const [editingAssistant, setEditingAssistant] = useState<Assistant | null>(
    null,
  )
  const [cloneSource, setCloneSource] = useState<Assistant | null>(null)
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
      const response = await ApiClient.Assistant.list({
        page: 1,
        per_page: 100,
      })
      // Filter out template assistants for user view
      const userAssistants = response.assistants.filter(a => !a.is_template)
      const templateAssistants = response.assistants.filter(a => a.is_template)

      setAssistants(userAssistants)
      setTemplateAssistants(templateAssistants)
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
        is_active: values.is_active,
      }

      if (editingAssistant) {
        await ApiClient.Assistant.update({
          assistant_id: editingAssistant.id,
          ...requestData,
        })
        message.success('Assistant updated successfully')
      } else if (cloneSource) {
        await ApiClient.Assistant.create({
          ...requestData,
          // Note: clone_from is not supported by the API yet
        })
        message.success('Assistant cloned successfully')
      } else {
        await ApiClient.Assistant.create(requestData)
        message.success('Assistant created successfully')
      }

      setModalVisible(false)
      setEditingAssistant(null)
      setCloneSource(null)
      form.resetFields()
      fetchAssistants()
    } catch (error) {
      message.error('Failed to save assistant')
    }
  }

  const handleDelete = async (assistant: Assistant) => {
    try {
      await ApiClient.Assistant.delete({ assistant_id: assistant.id })
      message.success('Assistant deleted successfully')
      fetchAssistants()
    } catch (error) {
      message.error('Failed to delete assistant')
    }
  }

  const handleEdit = (assistant: Assistant) => {
    setEditingAssistant(assistant)
    setCloneSource(null)
    const parametersJson = assistant.parameters
      ? JSON.stringify(assistant.parameters, null, 2)
      : ''
    form.setFieldsValue({
      name: assistant.name,
      description: assistant.description,
      instructions: assistant.instructions,
      parameters: parametersJson,
      is_active: assistant.is_active,
    })
    setParameterJson(parametersJson)
    convertJsonToForm(parametersJson)
    setParameterMode('json')
    setModalVisible(true)
  }

  const handleCreate = () => {
    setEditingAssistant(null)
    setCloneSource(null)
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
      is_active: true,
      parameters: defaultParams,
    })
    setParameterJson(defaultParams)
    convertJsonToForm(defaultParams)
    setParameterMode('json')
    setModalVisible(true)
  }

  const handleCloneFromTemplate = () => {
    setTemplateModalVisible(true)
  }

  const handleSelectTemplateAssistant = (assistant: Assistant) => {
    setCloneSource(assistant)
    setEditingAssistant(null)
    const parametersJson = assistant.parameters
      ? JSON.stringify(assistant.parameters, null, 2)
      : ''
    form.setFieldsValue({
      name: `${assistant.name} (Copy)`,
      description: assistant.description,
      instructions: assistant.instructions,
      parameters: parametersJson,
      is_active: true,
    })
    setParameterJson(parametersJson)
    convertJsonToForm(parametersJson)
    setTemplateModalVisible(false)
    setModalVisible(true)
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

  const renderAssistantCard = (assistant: Assistant) => (
    <Card
      key={assistant.id}
      hoverable
      actions={[
        <Tooltip title="Edit" key="edit">
          <EditOutlined onClick={() => handleEdit(assistant)} />
        </Tooltip>,
        <Popconfirm
          title="Delete Assistant"
          description="Are you sure you want to delete this assistant?"
          onConfirm={() => handleDelete(assistant)}
          okText="Yes"
          cancelText="No"
          key="delete"
        >
          <Tooltip title="Delete">
            <DeleteOutlined />
          </Tooltip>
        </Popconfirm>,
      ]}
    >
      <Card.Meta
        avatar={
          <Avatar
            size={48}
            icon={<RobotOutlined />}
            style={{ backgroundColor: '#1890ff' }}
          />
        }
        title={
          <div className="flex items-center gap-2">
            <Text strong>{assistant.name}</Text>
            <Tag color="green">{t('assistants.personal')}</Tag>
            {!assistant.is_active && <Tag color="red">{t('assistants.inactive')}</Tag>}
          </div>
        }
        description={
          <div>
            <Text type="secondary" className="block mb-2">
              {assistant.description || 'No description'}
            </Text>
            <Text type="secondary" className="text-xs">
              Created {new Date(assistant.created_at).toLocaleDateString()}
            </Text>
          </div>
        }
      />
    </Card>
  )

  return (
    <PageContainer>
        <Row gutter={[24, 24]}>
          <Col span={24}>
            <div className="flex justify-between items-center mb-6">
              <div>
                <Title level={2}>{t('assistants.title')}</Title>
                <Text type="secondary">
                  {t('assistants.subtitle')}
                </Text>
              </div>
              <Space>
                <Button
                  type="default"
                  icon={<CopyOutlined />}
                  onClick={handleCloneFromTemplate}
                >
                  Clone from Template
                </Button>
                <Button
                  type="primary"
                  icon={<PlusOutlined />}
                  onClick={handleCreate}
                >
                  Create New
                </Button>
              </Space>
            </div>

            {loading ? (
              <div className="flex justify-center items-center py-12">
                <div>Loading assistants...</div>
              </div>
            ) : assistants.length === 0 ? (
              <Card>
                <div className="text-center py-12">
                  <RobotOutlined className="text-4xl mb-4" />
                  <Title level={4} type="secondary">
                    No assistants yet
                  </Title>
                  <Text type="secondary">
                    Create your first assistant to get started
                  </Text>
                </div>
              </Card>
            ) : (
              <Row gutter={[16, 16]}>
                {assistants.map(assistant => (
                  <Col xs={24} sm={12} md={8} lg={6} key={assistant.id}>
                    {renderAssistantCard(assistant)}
                  </Col>
                ))}
              </Row>
            )}
          </Col>
        </Row>

        <Modal
          title={
            editingAssistant
              ? 'Edit Assistant'
              : cloneSource
                ? 'Clone Assistant'
                : 'Create Assistant'
          }
          open={modalVisible}
          onCancel={() => {
            setModalVisible(false)
            setEditingAssistant(null)
            setCloneSource(null)
            form.resetFields()
          }}
          footer={null}
          width={800}
          maskClosable={false}
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

            <Form.Item name="is_active" label="Active" valuePropName="checked">
              <Switch />
            </Form.Item>

            <Form.Item>
              <Space>
                <Button type="primary" htmlType="submit">
                  {editingAssistant
                    ? 'Update'
                    : cloneSource
                      ? 'Clone'
                      : 'Create'}
                </Button>
                <Button
                  onClick={() => {
                    setModalVisible(false)
                    setEditingAssistant(null)
                    setCloneSource(null)
                    form.resetFields()
                  }}
                >
                  Cancel
                </Button>
              </Space>
            </Form.Item>
          </Form>
        </Modal>

        {/* Template Assistants Modal */}
        <Modal
          title="Clone from Template Assistants"
          open={templateModalVisible}
          onCancel={() => setTemplateModalVisible(false)}
          footer={null}
          width={900}
          maskClosable={false}
        >
          <div className="mb-4">
            <Text type="secondary">
              Select a template assistant to clone and customize for your use
            </Text>
          </div>
          <Table
            columns={[
              {
                title: 'Name',
                dataIndex: 'name',
                key: 'name',
                render: (text: string) => (
                  <Space>
                    <RobotOutlined />
                    <Text strong>{text}</Text>
                    <Tag color="blue">Template</Tag>
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
                title: 'Instructions Preview',
                dataIndex: 'instructions',
                key: 'instructions',
                render: (text: string) => (
                  <Text type="secondary" ellipsis={{ tooltip: text }}>
                    {text
                      ? text.substring(0, 100) +
                        (text.length > 100 ? '...' : '')
                      : 'No instructions'}
                  </Text>
                ),
                width: 200,
              },
              {
                title: 'Actions',
                key: 'actions',
                render: (_: any, record: Assistant) => (
                  <Space>
                    <Tooltip title="Preview Details">
                      <Button
                        type="text"
                        icon={<RobotOutlined />}
                        onClick={() => {
                          Modal.info({
                            title: `Preview: ${record.name}`,
                            content: (
                              <div>
                                <div className="mb-3">
                                  <Text strong>Description:</Text>
                                  <div>
                                    {record.description || 'No description'}
                                  </div>
                                </div>
                                <div className="mb-3">
                                  <Text strong>Instructions:</Text>
                                  <div style={{ whiteSpace: 'pre-wrap' }}>
                                    {record.instructions || 'No instructions'}
                                  </div>
                                </div>
                                <div className="mb-3">
                                  <Text strong>Parameters:</Text>
                                  <pre
                                    style={{
                                      backgroundColor: '#f5f5f5',
                                      padding: '8px',
                                      borderRadius: '4px',
                                    }}
                                  >
                                    {record.parameters
                                      ? JSON.stringify(
                                          record.parameters,
                                          null,
                                          2,
                                        )
                                      : 'No parameters'}
                                  </pre>
                                </div>
                              </div>
                            ),
                            width: 600,
                          })
                        }}
                      />
                    </Tooltip>
                    <Button
                      type="primary"
                      icon={<CopyOutlined />}
                      onClick={() => handleSelectTemplateAssistant(record)}
                    >
                      Clone
                    </Button>
                  </Space>
                ),
              },
            ]}
            dataSource={templateAssistants}
            rowKey="id"
            pagination={{ pageSize: 5 }}
          />
        </Modal>
    </PageContainer>
  )
}
