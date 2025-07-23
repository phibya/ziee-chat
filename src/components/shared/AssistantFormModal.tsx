import React, { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  Button,
  Form,
  Input,
  InputNumber,
  Modal,
  Select,
  Space,
  Switch,
  Tag,
  Tooltip,
  Typography,
} from 'antd'
import {
  CodeOutlined,
  DeleteOutlined,
  FormOutlined,
  PlusOutlined,
} from '@ant-design/icons'
import { Assistant } from '../../types/api/assistant'
import {
  useModalsUIStore,
  closeAssistantModal,
  setAssistantModalLoading,
} from '../../store/ui/modals'
import { createUserAssistant, updateUserAssistant } from '../../store'

const { Text } = Typography
const { TextArea } = Input

// Predefined parameters list for validation and suggestions
const PREDEFINED_PARAMETERS = [
  {
    name: 'stream',
    type: 'boolean',
    description: 'Enable streaming responses',
  },
  {
    name: 'temperature',
    type: 'number',
    description: 'Controls randomness in responses (0.0-2.0)',
  },
  {
    name: 'frequency_penalty',
    type: 'number',
    description: 'Reduces repetition of words (-2.0 to 2.0)',
  },
  {
    name: 'presence_penalty',
    type: 'number',
    description: 'Encourages new topics (-2.0 to 2.0)',
  },
  {
    name: 'top_p',
    type: 'number',
    description: 'Nucleus sampling parameter (0.0-1.0)',
  },
  { name: 'top_k', type: 'number', description: 'Top-k sampling parameter' },
  {
    name: 'max_tokens',
    type: 'number',
    description: 'Maximum tokens in response',
  },
  {
    name: 'seed',
    type: 'number',
    description: 'Random seed for reproducibility',
  },
  {
    name: 'stop',
    type: 'string',
    description: 'Stop sequences for generation',
  },
] as const

// Default parameter values
const DEFAULT_PARAMETER_VALUES: Record<string, any> = {
  stream: true,
  temperature: 0.7,
  frequency_penalty: 0.0,
  presence_penalty: 0.0,
  top_p: 0.95,
  top_k: 40,
  max_tokens: 2048,
  seed: null,
  stop: '',
}

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

export const AssistantFormModal: React.FC = () => {
  const { t } = useTranslation()
  const [form] = Form.useForm<AssistantFormData>()
  const [parameterMode, setParameterMode] = useState<'json' | 'form'>('json')
  const [parameterFormFields, setParameterFormFields] = useState<
    ParameterFormField[]
  >([])
  const [parameterJson, setParameterJson] = useState('')
  const [jsonErrors, setJsonErrors] = useState<string[]>([])

  // Store usage
  const { assistantModalOpen, assistantModalLoading, editingAssistant } =
    useModalsUIStore()

  // No store state needed, using external methods

  // TODO: Handle clone source through store if needed
  const cloneSource: Assistant | null = null
  const isAdmin = false // TODO: Handle admin flag through auth store

  // Validation functions
  const validateParameterName = (name: string): boolean => {
    return PREDEFINED_PARAMETERS.some(param => param.name === name)
  }

  const validateJsonParameters = (jsonString: string): string[] => {
    const errors: string[] = []

    if (!jsonString.trim()) return errors

    try {
      const parsed = JSON.parse(jsonString)
      const keys = Object.keys(parsed)

      // Check for duplicate keys (JSON.parse handles this, but we check for user awareness)
      const uniqueKeys = new Set(keys)
      if (keys.length !== uniqueKeys.size) {
        errors.push('Duplicate parameter names detected')
      }

      // Check for invalid parameter names
      const invalidParams = keys.filter(key => !validateParameterName(key))
      if (invalidParams.length > 0) {
        errors.push(`Invalid parameter names: ${invalidParams.join(', ')}`)
      }
    } catch (_error) {
      errors.push('Invalid JSON format')
    }

    return errors
  }

  const getAvailableParameters =
    (): (typeof PREDEFINED_PARAMETERS)[number][] => {
      const currentParams = new Set()

      if (parameterMode === 'json' && parameterJson) {
        try {
          const parsed = JSON.parse(parameterJson)
          Object.keys(parsed).forEach(key => currentParams.add(key))
        } catch {
          // Do nothing
        }
      } else if (parameterMode === 'form') {
        parameterFormFields.forEach(field => {
          if (field.name.trim()) currentParams.add(field.name)
        })
      }

      return PREDEFINED_PARAMETERS.filter(
        param => !currentParams.has(param.name),
      )
    }

  const getIncludedParameters =
    (): (typeof PREDEFINED_PARAMETERS)[number][] => {
      const currentParams = new Set()

      if (parameterMode === 'json' && parameterJson) {
        try {
          const parsed = JSON.parse(parameterJson)
          Object.keys(parsed).forEach(key => currentParams.add(key))
        } catch {
          // Do nothing
        }
      } else if (parameterMode === 'form') {
        parameterFormFields.forEach(field => {
          if (field.name.trim()) currentParams.add(field.name)
        })
      }

      return PREDEFINED_PARAMETERS.filter(param =>
        currentParams.has(param.name),
      )
    }

  const addParameterFromTag = (paramName: string) => {
    const paramDef = PREDEFINED_PARAMETERS.find(p => p.name === paramName)
    if (!paramDef) return

    const defaultValue =
      DEFAULT_PARAMETER_VALUES[paramName] ??
      (paramDef.type === 'boolean' ? true : paramDef.type === 'number' ? 0 : '')

    if (parameterMode === 'json') {
      try {
        const parsed = parameterJson ? JSON.parse(parameterJson) : {}
        parsed[paramName] = defaultValue
        const newJson = JSON.stringify(parsed, null, 2)
        setParameterJson(newJson)
        form.setFieldsValue({ parameters: newJson })
      } catch {
        // Handle invalid JSON
        const newParam = { [paramName]: defaultValue }
        const newJson = JSON.stringify(newParam, null, 2)
        setParameterJson(newJson)
        form.setFieldsValue({ parameters: newJson })
      }
    } else {
      setParameterFormFields([
        ...parameterFormFields,
        { name: paramName, type: paramDef.type as any, value: defaultValue },
      ])
    }
  }

  const removeParameterFromTag = (paramName: string) => {
    if (parameterMode === 'json') {
      try {
        const parsed = JSON.parse(parameterJson)
        delete parsed[paramName]
        const newJson = JSON.stringify(parsed, null, 2)
        setParameterJson(newJson)
        form.setFieldsValue({ parameters: newJson })
      } catch {
        // If JSON is invalid, just clear it
        setParameterJson('{}')
        form.setFieldsValue({ parameters: '{}' })
      }
    } else {
      const newFields = parameterFormFields.filter(
        field => field.name !== paramName,
      )
      setParameterFormFields(newFields)
    }
  }

  // Real-time validation for JSON
  useEffect(() => {
    if (parameterMode === 'json') {
      const errors = validateJsonParameters(parameterJson)
      setJsonErrors(errors)
    }
  }, [parameterJson, parameterMode])

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

  const handleSubmit = async (values: AssistantFormData) => {
    // Get parameters from current mode
    let parametersString = values.parameters
    if (parameterMode === 'form') {
      parametersString = convertFormToJson(parameterFormFields)
    }

    let parametersObject = {}
    try {
      parametersObject = parametersString ? JSON.parse(parametersString) : {}
    } catch (error) {
      console.error('Invalid JSON in parameters:', error)
      parametersObject = {}
    }

    const finalValues = {
      ...values,
      parameters: parametersObject,
    }

    setAssistantModalLoading(true)
    try {
      if (editingAssistant) {
        await updateUserAssistant(editingAssistant.id, finalValues)
      } else {
        await createUserAssistant(finalValues)
      }
      closeAssistantModal()
    } catch (error) {
      console.error('Failed to save assistant:', error)
    } finally {
      setAssistantModalLoading(false)
    }
  }

  // Initialize form when modal opens or editing assistant changes
  useEffect(() => {
    if (assistantModalOpen) {
      if (editingAssistant) {
        // Editing existing assistant
        const parametersJson = editingAssistant.parameters
          ? JSON.stringify(editingAssistant.parameters, null, 2)
          : ''
        form.setFieldsValue({
          name: editingAssistant.name,
          description: editingAssistant.description,
          instructions: editingAssistant.instructions,
          parameters: parametersJson,
          is_active: editingAssistant.is_active,
        })
        setParameterJson(parametersJson)
        convertJsonToForm(parametersJson)
      } else {
        // Creating new assistant
        const defaultParams = JSON.stringify({ stream: true }, null, 2)
        form.setFieldsValue({
          is_active: true,
          parameters: defaultParams,
        })
        setParameterJson(defaultParams)
        convertJsonToForm(defaultParams)
      }
      setParameterMode('json')
    } else {
      // Reset when modal closes
      form.resetFields()
      setParameterJson('')
      setParameterFormFields([])
      setJsonErrors([])
    }
  }, [assistantModalOpen, editingAssistant, cloneSource, form])

  const getTitle = () => {
    if (editingAssistant) {
      return isAdmin ? 'Edit Template Assistant' : 'Edit Assistant'
    }
    return isAdmin ? 'Create Template Assistant' : 'Create Assistant'
  }

  return (
    <Modal
      title={getTitle()}
      open={assistantModalOpen}
      onCancel={closeAssistantModal}
      footer={null}
      width={800}
      maskClosable={false}
    >
      <Form form={form} onFinish={handleSubmit} layout="vertical">
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

          {/* Parameter Tags */}
          <div className="mb-4">
            <div className="mb-2">
              <Text strong>Included Parameters:</Text>
            </div>
            <div className="mb-2 flex gap-1 flex-wrap">
              {getIncludedParameters().map(param => (
                <Tag
                  key={param.name}
                  color="green"
                  onClick={() => removeParameterFromTag(param.name)}
                  className="cursor-pointer"
                >
                  <Tooltip title={param.description}>
                    × {param.name} ({param.type})
                  </Tooltip>
                </Tag>
              ))}
              {getIncludedParameters().length === 0 && (
                <Text type="secondary">No parameters added yet</Text>
              )}
            </div>

            <div className="mb-2">
              <Text strong>Available Parameters:</Text>
            </div>
            <div className="flex gap-1 flex-wrap">
              {getAvailableParameters().map(param => (
                <Tag
                  key={param.name}
                  color="blue"
                  className="cursor-pointer"
                  onClick={() => addParameterFromTag(param.name)}
                >
                  <Tooltip title={param.description}>
                    + {param.name} ({param.type})
                  </Tooltip>
                </Tag>
              ))}
              {getAvailableParameters().length === 0 && (
                <Text type="secondary">All parameters are included</Text>
              )}
            </div>
          </div>

          {parameterMode === 'json' ? (
            <Form.Item
              name="parameters"
              rules={[
                {
                  validator: (_, value) => {
                    if (!value) return Promise.resolve()
                    const errors = validateJsonParameters(value)
                    if (errors.length > 0) {
                      return Promise.reject(errors.join(', '))
                    }
                    return Promise.resolve()
                  },
                },
              ]}
              validateStatus={jsonErrors.length > 0 ? 'error' : ''}
              help={jsonErrors.length > 0 ? jsonErrors.join(', ') : ''}
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
                      placeholder={t('forms.fieldName')}
                      value={field.name}
                      onChange={e =>
                        handleFormFieldChange(index, 'name', e.target.value)
                      }
                      style={{ width: 150 }}
                      status={
                        field.name && !validateParameterName(field.name)
                          ? 'error'
                          : ''
                      }
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
                          handleFormFieldChange(index, 'value', e.target.value)
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
          name="is_active"
          label={t('labels.active')}
          valuePropName="checked"
        >
          <Switch />
        </Form.Item>

        <Form.Item>
          <Space>
            <Button
              type="primary"
              htmlType="submit"
              loading={assistantModalLoading}
            >
              {editingAssistant ? 'Update' : 'Create'}
            </Button>
            <Button
              onClick={closeAssistantModal}
              disabled={assistantModalLoading}
            >
              Cancel
            </Button>
          </Space>
        </Form.Item>
      </Form>
    </Modal>
  )
}
