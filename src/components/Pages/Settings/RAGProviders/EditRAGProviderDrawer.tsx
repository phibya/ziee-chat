import { EyeInvisibleOutlined, EyeTwoTone } from '@ant-design/icons'
import { Button, Card, Flex, Form, Input, Switch } from 'antd'
import { Drawer } from '../../../common/Drawer'
import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import {
  closeEditRAGProviderDrawer,
  setEditRAGProviderDrawerLoading,
  Stores,
  updateRAGProvider,
} from '../../../../store'
import {
  UpdateRAGProviderRequest,
  RAGProviderType,
} from '../../../../types/api'

export function EditRAGProviderDrawer() {
  const { t } = useTranslation()
  const [form] = Form.useForm()

  const { open, loading, providerId } = Stores.UI.EditRAGProviderDrawer
  const { providers } = Stores.AdminRAGProviders

  // Find the current provider from the store
  const provider = providerId ? providers.find(p => p.id === providerId) : null

  useEffect(() => {
    if (provider && open) {
      form.setFieldsValue({
        name: provider.name,
        enabled: provider.enabled,
        api_key: provider.api_key,
        base_url: provider.base_url,
      })
    }
  }, [provider, open, form])

  const handleSubmit = async () => {
    if (!provider) return

    try {
      setEditRAGProviderDrawerLoading(true)
      const values = await form.validateFields()
      await updateRAGProvider(provider.id, values as UpdateRAGProviderRequest)
      closeEditRAGProviderDrawer()
    } catch (error) {
      console.error('Failed to update RAG provider:', error)
    } finally {
      setEditRAGProviderDrawerLoading(false)
    }
  }

  if (!provider) return null

  return (
    <Drawer
      title={`Edit RAG Provider: ${provider.name}`}
      open={open}
      onClose={closeEditRAGProviderDrawer}
      footer={[
        <Button key="cancel" onClick={closeEditRAGProviderDrawer}>
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
      <Form form={form} layout="vertical">
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

        <Form.Item name="enabled" label="Enabled" valuePropName="checked">
          <Switch />
        </Form.Item>

        {/* API Configuration for non-local providers */}
        {provider.type !== 'local' && (
          <Flex vertical className="gap-2 w-full">
            <Card title="API Configuration">
              <Form.Item name="api_key" label="API Key">
                <Input.Password
                  placeholder="Enter API key"
                  iconRender={visible =>
                    visible ? <EyeTwoTone /> : <EyeInvisibleOutlined />
                  }
                />
              </Form.Item>

              <Form.Item
                name="base_url"
                label="Base URL"
                rules={[
                  {
                    required: provider.type !== ('local' as RAGProviderType),
                    message: 'Base URL is required for remote providers',
                  },
                ]}
              >
                <Input placeholder="Enter base URL" />
              </Form.Item>
            </Card>
          </Flex>
        )}
      </Form>
    </Drawer>
  )
}
