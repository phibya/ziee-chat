import { Card, Form, Input, Modal, Space, Switch } from 'antd'
import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { EyeInvisibleOutlined, EyeTwoTone } from '@ant-design/icons'
import { Provider, UpdateProviderRequest } from '../../../../types/api/provider'

interface EditProviderModalProps {
  open: boolean
  provider: Provider | null
  onClose: () => void
  onSubmit: (provider: UpdateProviderRequest) => void
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
      })
    }
  }, [provider, open, form])

  const handleSubmit = async () => {
    try {
      const values = await form.validateFields()
      onSubmit({
        id: provider!.id,
        ...values,
      } as UpdateProviderRequest)
    } catch (error) {
      console.error('Form validation failed:', error)
    }
  }

  if (!provider) return null

  return (
    <Modal
      title={`${t('providers.editProvider')} ${provider.name}`}
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
          name="enabled"
          label={t('providers.enabled')}
          valuePropName="checked"
        >
          <Switch />
        </Form.Item>

        {/* API Configuration for non-local providers */}
        {provider.type !== 'local' && (
          <Space direction="vertical" size="middle" style={{ width: '100%' }}>
            <Card size="small" title={t('providers.apiConfiguration')}>
              <Form.Item
                name="api_key"
                label={t('providers.apiKey')}
                rules={[
                  {
                    required: true,
                    message: t('providers.apiKeyRequired'),
                  },
                ]}
              >
                <Input.Password
                  placeholder={t('providers.apiKeyPlaceholder')}
                  iconRender={visible =>
                    visible ? <EyeTwoTone /> : <EyeInvisibleOutlined />
                  }
                />
              </Form.Item>

              <Form.Item
                name="base_url"
                label={t('providers.baseUrl')}
                rules={[
                  {
                    required: true,
                    message: t('providers.baseUrlRequired'),
                  },
                ]}
              >
                <Input placeholder={t('providers.baseUrlPlaceholder')} />
              </Form.Item>
            </Card>
          </Space>
        )}
      </Form>
    </Modal>
  )
}
