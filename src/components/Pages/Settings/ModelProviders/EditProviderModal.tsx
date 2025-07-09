import {
  Card,
  Form,
  Input,
  InputNumber,
  Modal,
  Select,
  Space,
  Switch,
} from 'antd'
import { useEffect } from 'react'
import { EyeInvisibleOutlined, EyeTwoTone } from '@ant-design/icons'
import {
  ModelProvider,
  UpdateModelProviderRequest,
} from '../../../../types/api/modelProvider'

interface EditProviderModalProps {
  open: boolean
  provider: ModelProvider | null
  onClose: () => void
  onSubmit: (provider: UpdateModelProviderRequest) => void
  loading?: boolean
}

export function EditProviderModal({
  open,
  provider,
  onClose,
  onSubmit,
  loading,
}: EditProviderModalProps) {
  const [form] = Form.useForm()

  useEffect(() => {
    if (provider && open) {
      form.setFieldsValue({
        name: provider.name,
        enabled: provider.enabled,
        api_key: provider.api_key,
        base_url: provider.base_url,
        settings: provider.settings,
      })
    }
  }, [provider, open, form])

  const handleSubmit = async () => {
    try {
      const values = await form.validateFields()
      onSubmit({
        id: provider!.id,
        ...values,
      } as UpdateModelProviderRequest)
    } catch (error) {
      console.error('Form validation failed:', error)
    }
  }

  if (!provider) return null

  return (
    <Modal
      title={`Edit ${provider.name}`}
      open={open}
      onCancel={onClose}
      onOk={handleSubmit}
      confirmLoading={loading}
      width={600}
      destroyOnClose
      maskClosable={false}
    >
      <Form form={form} layout="vertical">
        <Form.Item
          name="name"
          label="Provider Name"
          rules={[{ required: true, message: 'Please enter provider name' }]}
        >
          <Input placeholder="Enter provider name" />
        </Form.Item>

        <Form.Item name="enabled" label="Enabled" valuePropName="checked">
          <Switch />
        </Form.Item>

        {/* API Configuration for non-llama.cpp providers */}
        {provider.type !== 'llama.cpp' && (
          <Space direction="vertical" size="middle" style={{ width: '100%' }}>
            <Card size="small" title="API Configuration">
              <Form.Item
                name="api_key"
                label="API Key"
                rules={[{ required: true, message: 'Please enter API key' }]}
              >
                <Input.Password
                  placeholder="Insert API Key"
                  iconRender={visible =>
                    visible ? <EyeTwoTone /> : <EyeInvisibleOutlined />
                  }
                />
              </Form.Item>

              <Form.Item
                name="base_url"
                label="Base URL"
                rules={[{ required: true, message: 'Please enter base URL' }]}
              >
                <Input placeholder="Base URL" />
              </Form.Item>
            </Card>
          </Space>
        )}

        {/* Llama.cpp Configuration */}
        {provider.type === 'llama.cpp' && (
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

              <Form.Item name={['settings', 'cpuThreads']} label="CPU Threads">
                <InputNumber
                  placeholder="-1 (auto)"
                  style={{ width: '100%' }}
                />
              </Form.Item>

              <Form.Item
                name={['settings', 'threadsBatch']}
                label="Threads (Batch)"
              >
                <InputNumber
                  placeholder="-1 (same as Threads)"
                  style={{ width: '100%' }}
                />
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
