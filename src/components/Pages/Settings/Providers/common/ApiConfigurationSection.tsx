import { Card, Form, Input } from 'antd'
import { useTranslation } from 'react-i18next'
import { EyeInvisibleOutlined, EyeTwoTone } from '@ant-design/icons'

export function ApiConfigurationSection() {
  const { t } = useTranslation()

  return (
    <Card title={t('providers.apiConfiguration')}>
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
  )
}
