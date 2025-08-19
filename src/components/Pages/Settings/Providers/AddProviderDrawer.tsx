import { Button, Form, Input, Select, Switch } from 'antd'
import { Drawer } from '../../../common/Drawer'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  PROVIDER_DEFAULTS,
  SUPPORTED_PROVIDERS,
} from '../../../../constants/providers'
import {
  closeAddProviderDrawer,
  createNewModelProvider,
  setAddProviderDrawerLoading,
  Stores,
} from '../../../../store'
import { CreateProviderRequest, ProviderType } from '../../../../types'
import { ApiConfigurationSection } from './common'

export function AddProviderDrawer() {
  const { t } = useTranslation()
  const [form] = Form.useForm()
  const [providerType, setProviderType] = useState<ProviderType>('local')

  const { open, loading } = Stores.UI.AddProviderDrawer

  // No store state needed for this component

  const handleSubmit = async () => {
    try {
      const values = await form.validateFields()
      setAddProviderDrawerLoading(true)
      await createNewModelProvider(values as CreateProviderRequest)
      closeAddProviderDrawer()
    } catch (error) {
      console.error('Failed to create provider:', error)
    } finally {
      setAddProviderDrawerLoading(false)
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
    <Drawer
      title={t('providers.addProviderTitle')}
      open={open}
      onClose={closeAddProviderDrawer}
      footer={[
        <Button key="cancel" onClick={closeAddProviderDrawer}>
          {t('buttons.cancel')}
        </Button>,
        <Button
          key="submit"
          type="primary"
          loading={loading}
          onClick={handleSubmit}
        >
          {t('buttons.ok')}
        </Button>,
      ]}
      width={400}
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
    </Drawer>
  )
}
