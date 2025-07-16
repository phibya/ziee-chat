import { Form, Input, Modal, Select, Switch } from 'antd'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  CreateProviderRequest,
  ProviderType,
} from '../../../../types/api/modelProvider'
import {
  PROVIDER_DEFAULTS,
  SUPPORTED_PROVIDERS,
} from '../../../../constants/modelProviders'
import { ApiConfigurationSection, CandleConfigurationSection } from './shared'

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
  const [providerType, setProviderType] = useState<ProviderType>('candle')

  const handleSubmit = async () => {
    try {
      const values = await form.validateFields()
      onSubmit(values as CreateModelProviderRequest)
    } catch (error) {
      console.error('Form validation failed:', error)
    }
  }

  const handleTypeChange = (type: ProviderType) => {
    setProviderType(type)
    // Reset form when type changes
    form.resetFields(['api_key', 'base_url', 'settings'])

    // Set default values based on provider type
    const defaults = getDefaultValues(type)
    form.setFieldsValue(defaults)
  }

  const getDefaultValues = (type: ProviderType) => {
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
          type: 'candle',
          enabled: true,
          ...getDefaultValues('candle'),
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

        {/* API Configuration for non-candle providers */}
        {providerType !== 'candle' && <ApiConfigurationSection />}

        {/* Candle Configuration */}
        {providerType === 'candle' && <CandleConfigurationSection />}
      </Form>
    </Modal>
  )
}
