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
import { useTranslation } from 'react-i18next'
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
  const { t } = useTranslation()
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
      title={`${t('modelProviders.editProvider')} ${provider.name}`}
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
          label={t('modelProviders.providerName')}
          rules={[
            {
              required: true,
              message: t('modelProviders.providerNameRequired'),
            },
          ]}
        >
          <Input placeholder={t('modelProviders.providerNamePlaceholder')} />
        </Form.Item>

        <Form.Item
          name="enabled"
          label={t('modelProviders.enabled')}
          valuePropName="checked"
        >
          <Switch />
        </Form.Item>

        {/* API Configuration for non-llama.cpp providers */}
        {provider.type !== 'llama.cpp' && (
          <Space direction="vertical" size="middle" style={{ width: '100%' }}>
            <Card size="small" title={t('modelProviders.apiConfiguration')}>
              <Form.Item
                name="api_key"
                label={t('modelProviders.apiKey')}
                rules={[
                  {
                    required: true,
                    message: t('modelProviders.apiKeyRequired'),
                  },
                ]}
              >
                <Input.Password
                  placeholder={t('modelProviders.apiKeyPlaceholder')}
                  iconRender={visible =>
                    visible ? <EyeTwoTone /> : <EyeInvisibleOutlined />
                  }
                />
              </Form.Item>

              <Form.Item
                name="base_url"
                label={t('modelProviders.baseUrl')}
                rules={[
                  {
                    required: true,
                    message: t('modelProviders.baseUrlRequired'),
                  },
                ]}
              >
                <Input placeholder={t('modelProviders.baseUrlPlaceholder')} />
              </Form.Item>
            </Card>
          </Space>
        )}

        {/* Llama.cpp Configuration */}
        {provider.type === 'llama.cpp' && (
          <Card size="small" title={t('modelProviders.llamaCppConfiguration')}>
            <Space direction="vertical" size="middle" style={{ width: '100%' }}>
              <Form.Item
                name={['settings', 'autoUnloadOldModels']}
                label={t('modelProviders.autoUnloadOldModels')}
                valuePropName="checked"
              >
                <Switch />
              </Form.Item>

              <Form.Item
                name={['settings', 'contextShift']}
                label={t('modelProviders.contextShift')}
                valuePropName="checked"
              >
                <Switch />
              </Form.Item>

              <Form.Item
                name={['settings', 'continuousBatching']}
                label={t('modelProviders.continuousBatching')}
                valuePropName="checked"
              >
                <Switch />
              </Form.Item>

              <Form.Item
                name={['settings', 'parallelOperations']}
                label={t('modelProviders.parallelOperations')}
              >
                <InputNumber min={1} max={16} style={{ width: '100%' }} />
              </Form.Item>

              <Form.Item
                name={['settings', 'cpuThreads']}
                label={t('modelProviders.cpuThreads')}
              >
                <InputNumber
                  placeholder={t('modelProviders.cpuThreadsPlaceholder')}
                  style={{ width: '100%' }}
                />
              </Form.Item>

              <Form.Item
                name={['settings', 'threadsBatch']}
                label={t('modelProviders.threadsBatch')}
              >
                <InputNumber
                  placeholder={t('modelProviders.threadsBatchPlaceholder')}
                  style={{ width: '100%' }}
                />
              </Form.Item>

              <Form.Item
                name={['settings', 'flashAttention']}
                label={t('modelProviders.flashAttention')}
                valuePropName="checked"
              >
                <Switch />
              </Form.Item>

              <Form.Item
                name={['settings', 'caching']}
                label={t('modelProviders.caching')}
                valuePropName="checked"
              >
                <Switch />
              </Form.Item>

              <Form.Item
                name={['settings', 'kvCacheType']}
                label={t('modelProviders.kvCacheType')}
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
                label={t('modelProviders.mmap')}
                valuePropName="checked"
              >
                <Switch />
              </Form.Item>

              <Form.Item
                name={['settings', 'huggingFaceAccessToken']}
                label={t('modelProviders.huggingFaceAccessToken')}
              >
                <Input.Password
                  placeholder={t('modelProviders.huggingFaceTokenPlaceholder')}
                />
              </Form.Item>
            </Space>
          </Card>
        )}
      </Form>
    </Modal>
  )
}
