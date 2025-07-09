import {
  Modal,
  Form,
  Input,
  Select,
  Switch,
  Space,
  Card,
  InputNumber,
} from 'antd'
import { useState } from 'react'
import { EyeInvisibleOutlined, EyeTwoTone } from '@ant-design/icons'
import { ModelProviderType, CreateModelProviderRequest } from '../../../../types/api/modelProvider'


interface AddProviderModalProps {
  open: boolean
  onClose: () => void
  onSubmit: (provider: CreateModelProviderRequest) => void
  loading?: boolean
}

export function AddProviderModal({ open, onClose, onSubmit, loading }: AddProviderModalProps) {
  const [form] = Form.useForm()
  const [providerType, setProviderType] = useState<ModelProviderType>('llama.cpp')

  const handleSubmit = async () => {
    try {
      const values = await form.validateFields()
      onSubmit(values as CreateModelProviderRequest)
    } catch (error) {
      console.error('Form validation failed:', error)
    }
  }

  const handleTypeChange = (type: ModelProviderType) => {
    setProviderType(type)
    // Reset form when type changes
    form.resetFields(['apiKey', 'baseUrl', 'settings'])
    
    // Set default values based on provider type
    const defaults = getDefaultValues(type)
    form.setFieldsValue(defaults)
  }

  const getDefaultValues = (type: ModelProviderType) => {
    switch (type) {
      case 'openai':
        return { baseUrl: 'https://api.openai.com/v1' }
      case 'anthropic':
        return { baseUrl: 'https://api.anthropic.com/v1' }
      case 'groq':
        return { baseUrl: 'https://api.groq.com/openai/v1' }
      case 'gemini':
        return { baseUrl: 'https://generativelanguage.googleapis.com/v1beta/openai' }
      case 'mistral':
        return { baseUrl: 'https://api.mistral.ai' }
      case 'llama.cpp':
        return {
          settings: {
            autoUnloadOldModels: true,
            contextShift: false,
            continuousBatching: false,
            parallelOperations: 1,
            cpuThreads: -1,
            threadsBatch: -1,
            flashAttention: true,
            caching: true,
            kvCacheType: 'q8_0',
            mmap: true,
            huggingFaceAccessToken: '',
          }
        }
      default:
        return {}
    }
  }

  const providerTypeOptions = [
    { value: 'llama.cpp', label: 'Llama.cpp' },
    { value: 'openai', label: 'OpenAI' },
    { value: 'anthropic', label: 'Anthropic' },
    { value: 'groq', label: 'Groq' },
    { value: 'gemini', label: 'Gemini' },
    { value: 'mistral', label: 'Mistral' },
    { value: 'custom', label: 'Custom' },
  ]

  return (
    <Modal
      title="Add Model Provider"
      open={open}
      onCancel={onClose}
      onOk={handleSubmit}
      confirmLoading={loading}
      width={600}
      destroyOnClose
    >
      <Form
        form={form}
        layout="vertical"
        initialValues={{
          type: 'llama.cpp',
          enabled: true,
          ...getDefaultValues('llama.cpp')
        }}
      >
        <Form.Item
          name="name"
          label="Provider Name"
          rules={[{ required: true, message: 'Please enter provider name' }]}
        >
          <Input placeholder="Enter provider name" />
        </Form.Item>

        <Form.Item
          name="type"
          label="Provider Type"
          rules={[{ required: true, message: 'Please select provider type' }]}
        >
          <Select
            options={providerTypeOptions}
            onChange={handleTypeChange}
            placeholder="Select provider type"
          />
        </Form.Item>

        <Form.Item
          name="enabled"
          label="Enabled"
          valuePropName="checked"
        >
          <Switch />
        </Form.Item>

        {/* API Configuration for non-llama.cpp providers */}
        {providerType !== 'llama.cpp' && (
          <Space direction="vertical" size="middle" style={{ width: '100%' }}>
            <Card size="small" title="API Configuration">
              <Form.Item
                name="apiKey"
                label="API Key"
                rules={[{ required: true, message: 'Please enter API key' }]}
              >
                <Input.Password
                  placeholder="Insert API Key"
                  iconRender={visible => (visible ? <EyeTwoTone /> : <EyeInvisibleOutlined />)}
                />
              </Form.Item>

              <Form.Item
                name="baseUrl"
                label="Base URL"
                rules={[{ required: true, message: 'Please enter base URL' }]}
              >
                <Input placeholder="Base URL" />
              </Form.Item>
            </Card>
          </Space>
        )}

        {/* Llama.cpp Configuration */}
        {providerType === 'llama.cpp' && (
          <Card size="small" title="Llama.cpp Configuration">
            <Space direction="vertical" size="middle" style={{ width: '100%' }}>
              <Form.Item
                name={['settings', 'autoUnloadOldModels']}
                label="Auto-Unload Old Models"
                valuePropName="checked"
              >
                <Switch />
              </Form.Item>

              <Form.Item
                name={['settings', 'contextShift']}
                label="Context Shift"
                valuePropName="checked"
              >
                <Switch />
              </Form.Item>

              <Form.Item
                name={['settings', 'continuousBatching']}
                label="Continuous Batching"
                valuePropName="checked"
              >
                <Switch />
              </Form.Item>

              <Form.Item
                name={['settings', 'parallelOperations']}
                label="Parallel Operations"
              >
                <InputNumber min={1} max={16} style={{ width: '100%' }} />
              </Form.Item>

              <Form.Item
                name={['settings', 'cpuThreads']}
                label="CPU Threads"
              >
                <InputNumber placeholder="-1 (auto)" style={{ width: '100%' }} />
              </Form.Item>

              <Form.Item
                name={['settings', 'threadsBatch']}
                label="Threads (Batch)"
              >
                <InputNumber placeholder="-1 (same as Threads)" style={{ width: '100%' }} />
              </Form.Item>

              <Form.Item
                name={['settings', 'flashAttention']}
                label="Flash Attention"
                valuePropName="checked"
              >
                <Switch />
              </Form.Item>

              <Form.Item
                name={['settings', 'caching']}
                label="Caching"
                valuePropName="checked"
              >
                <Switch />
              </Form.Item>

              <Form.Item
                name={['settings', 'kvCacheType']}
                label="KV Cache Type"
              >
                <Select
                  options={[
                    { value: 'q8_0', label: 'q8_0' },
                    { value: 'q4_0', label: 'q4_0' },
                    { value: 'q4_1', label: 'q4_1' },
                    { value: 'q5_0', label: 'q5_0' },
                    { value: 'q5_1', label: 'q5_1' },
                  ]}
                />
              </Form.Item>

              <Form.Item
                name={['settings', 'mmap']}
                label="mmap"
                valuePropName="checked"
              >
                <Switch />
              </Form.Item>

              <Form.Item
                name={['settings', 'huggingFaceAccessToken']}
                label="Hugging Face Access Token"
              >
                <Input.Password placeholder="hf_*****************************" />
              </Form.Item>
            </Space>
          </Card>
        )}
      </Form>
    </Modal>
  )
}