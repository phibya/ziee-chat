import { Button, Form, Input, Select, Switch } from 'antd'
import { Drawer } from '../../../common/Drawer'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  RAG_PROVIDER_DEFAULTS,
  SUPPORTED_RAG_PROVIDERS,
} from '../../../../constants/ragProviders'
import {
  closeAddRAGProviderDrawer,
  createNewRAGProvider,
  setAddRAGProviderDrawerLoading,
  Stores,
} from '../../../../store'
import {
  CreateRAGProviderRequest,
  RAGProviderType,
} from '../../../../types/api'

export function AddRAGProviderDrawer() {
  const { t } = useTranslation()
  const [form] = Form.useForm()
  const [providerType, setProviderType] = useState<RAGProviderType>('local')

  const { open, loading } = Stores.UI.AddRAGProviderDrawer

  const handleSubmit = async () => {
    try {
      const values = await form.validateFields()
      setAddRAGProviderDrawerLoading(true)
      await createNewRAGProvider(values as CreateRAGProviderRequest)
      closeAddRAGProviderDrawer()
      form.resetFields()
    } catch (error) {
      console.error('Failed to create RAG provider:', error)
    } finally {
      setAddRAGProviderDrawerLoading(false)
    }
  }

  const handleTypeChange = (type: RAGProviderType) => {
    setProviderType(type)
    // Reset form when type changes
    form.resetFields(['api_key', 'base_url', 'settings'])

    // Set default values based on provider type
    const defaults = getDefaultValues(type)
    form.setFieldsValue(defaults)
  }

  const getDefaultValues = (type: RAGProviderType) => {
    return RAG_PROVIDER_DEFAULTS[type] || {}
  }

  const handleClose = () => {
    form.resetFields()
    closeAddRAGProviderDrawer()
  }

  return (
    <Drawer
      title="Add RAG Provider"
      open={open}
      onClose={handleClose}
      footer={[
        <Button key="cancel" onClick={handleClose}>
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
          label="Provider Name"
          rules={[
            {
              required: true,
              message: 'Provider name is required',
            },
          ]}
        >
          <Input placeholder="Enter provider name" />
        </Form.Item>

        <Form.Item
          name="type"
          label="Provider Type"
          rules={[
            {
              required: true,
              message: 'Provider type is required',
            },
          ]}
        >
          <Select
            options={SUPPORTED_RAG_PROVIDERS}
            onChange={handleTypeChange}
            placeholder="Select provider type"
          />
        </Form.Item>

        <Form.Item name="enabled" label="Enabled" valuePropName="checked">
          <Switch />
        </Form.Item>

        {/* API Configuration for non-local providers */}
        {providerType !== 'local' && (
          <>
            <Form.Item name="api_key" label="API Key">
              <Input.Password placeholder="Enter API key" />
            </Form.Item>

            <Form.Item
              name="base_url"
              label="Base URL"
              rules={[
                {
                  required: providerType !== ('local' as RAGProviderType),
                  message: 'Base URL is required for remote providers',
                },
              ]}
            >
              <Input placeholder="Enter base URL" />
            </Form.Item>
          </>
        )}
      </Form>
    </Drawer>
  )
}
