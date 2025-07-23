import { Form, Input, Modal, Select, Switch } from 'antd'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  CreateProviderRequest,
  ProviderType,
} from '../../../../types/api/provider'
import {
  PROVIDER_DEFAULTS,
  SUPPORTED_PROVIDERS,
} from '../../../../constants/providers'
import { ApiConfigurationSection } from './shared'
import { createNewModelProvider } from '../../../../store'
import {
  useModalsUIStore,
  closeAddProviderModal,
  setAddProviderModalLoading,
} from '../../../../store/ui/modals'

export function AddProviderModal() {
  const { t } = useTranslation()
  const [form] = Form.useForm()
  const [providerType, setProviderType] = useState<ProviderType>('local')

  const { addProviderModalOpen, addProviderModalLoading } = useModalsUIStore()

  // No store state needed for this component

  const handleSubmit = async () => {
    try {
      const values = await form.validateFields()
      setAddProviderModalLoading(true)
      await createNewModelProvider(values as CreateProviderRequest)
      closeAddProviderModal()
    } catch (error) {
      console.error('Failed to create provider:', error)
    } finally {
      setAddProviderModalLoading(false)
    }
  }

  const handleTypeChange = (type: ProviderType) => {
    setProviderType(type)
    // Reset form when type changes
    form.resetFields(['api_key', 'base_url'])

    // Set default values based on provider type
    const defaults = getDefaultValues(type)
    form.setFieldsValue(defaults)
  }

  const getDefaultValues = (type: ProviderType) => {
    return PROVIDER_DEFAULTS[type] || {}
  }

  return (
    <Modal
      title={t('providers.addProviderTitle')}
      open={addProviderModalOpen}
      onCancel={closeAddProviderModal}
      onOk={handleSubmit}
      confirmLoading={addProviderModalLoading}
      width={600}
      destroyOnClose
      maskClosable={false}
    >
      <Form
        form={form}
        layout="vertical"
        initialValues={{
          type: 'local',
          enabled: true,
          ...getDefaultValues('local'),
        }}
      >
        <Form.Item
          name="name"
          label={t('providers.providerName')}
          rules={[
            {
              required: true,
              message: t('providers.providerNameRequired'),
            },
          ]}
        >
          <Input placeholder={t('providers.providerNamePlaceholder')} />
        </Form.Item>

        <Form.Item
          name="type"
          label={t('providers.providerType')}
          rules={[
            {
              required: true,
              message: t('providers.providerTypeRequired'),
            },
          ]}
        >
          <Select
            options={SUPPORTED_PROVIDERS}
            onChange={handleTypeChange}
            placeholder={t('providers.providerTypePlaceholder')}
          />
        </Form.Item>

        <Form.Item
          name="enabled"
          label={t('providers.enabled')}
          valuePropName="checked"
        >
          <Switch />
        </Form.Item>

        {/* API Configuration for non-local providers */}
        {providerType !== 'local' && <ApiConfigurationSection />}
      </Form>
    </Modal>
  )
}
