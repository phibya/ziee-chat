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
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { EyeInvisibleOutlined, EyeTwoTone } from '@ant-design/icons'
import {
  CreateModelProviderRequest,
  ModelProviderType,
} from '../../../../types/api/modelProvider'
import {
  SUPPORTED_PROVIDERS,
  PROVIDER_DEFAULTS,
  KV_CACHE_TYPE_OPTIONS,
} from '../../../../constants/modelProviders'

interface AddProviderModalProps {
  open: boolean
  onClose: () => void
  onSubmit: (provider: CreateModelProviderRequest) => void
  loading?: boolean
}

export function AddProviderModal({
  open,
  onClose,
  onSubmit,
  loading,
}: AddProviderModalProps) {
  const { t } = useTranslation()
  const [form] = Form.useForm()
  const [providerType, setProviderType] =
    useState<ModelProviderType>('llama.cpp')

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
    form.resetFields(['api_key', 'base_url', 'settings'])

    // Set default values based on provider type
    const defaults = getDefaultValues(type)
    form.setFieldsValue(defaults)
  }

  const getDefaultValues = (type: ModelProviderType) => {
    return PROVIDER_DEFAULTS[type] || {}
  }

  return (
    <Modal
      title={t('modelProviders.addProviderTitle')}
      open={open}
      onCancel={onClose}
      onOk={handleSubmit}
      confirmLoading={loading}
      width={600}
      destroyOnClose
      maskClosable={false}
    >
      <Form
        form={form}
        layout="vertical"
        initialValues={{
          type: 'llama.cpp',
          enabled: true,
          ...getDefaultValues('llama.cpp'),
        }}
      >
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
          name="type"
          label={t('modelProviders.providerType')}
          rules={[
            {
              required: true,
              message: t('modelProviders.providerTypeRequired'),
            },
          ]}
        >
          <Select
            options={SUPPORTED_PROVIDERS}
            onChange={handleTypeChange}
            placeholder={t('modelProviders.providerTypePlaceholder')}
          />
        </Form.Item>

        <Form.Item
          name="enabled"
          label={t('modelProviders.enabled')}
          valuePropName="checked"
        >
          <Switch />
        </Form.Item>

        {/* API Configuration for non-llama.cpp providers */}
        {providerType !== 'llama.cpp' && (
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
        {providerType === 'llama.cpp' && (
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
                <Select options={KV_CACHE_TYPE_OPTIONS} />
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
