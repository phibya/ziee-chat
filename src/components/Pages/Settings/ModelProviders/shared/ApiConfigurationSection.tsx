import { Card, Form, Input } from 'antd'
import { useTranslation } from 'react-i18next'
import { EyeInvisibleOutlined, EyeTwoTone } from '@ant-design/icons'

interface ApiConfigurationSectionProps {
  showApiKey?: boolean
  showBaseUrl?: boolean
  apiKeyRequired?: boolean
  baseUrlRequired?: boolean
}

export function ApiConfigurationSection({
  showApiKey = true,
  showBaseUrl = true,
  apiKeyRequired = true,
  baseUrlRequired = true,
}: ApiConfigurationSectionProps) {
  const { t } = useTranslation()

  return (
    <Card size="small" title={t('modelProviders.apiConfiguration')}>
      {showApiKey && (
        <Form.Item
          name="api_key"
          label={t('modelProviders.apiKey')}
          rules={[
            {
              required: apiKeyRequired,
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
      )}

      {showBaseUrl && (
        <Form.Item
          name="base_url"
          label={t('modelProviders.baseUrl')}
          rules={[
            {
              required: baseUrlRequired,
              message: t('modelProviders.baseUrlRequired'),
            },
          ]}
        >
          <Input placeholder={t('modelProviders.baseUrlPlaceholder')} />
        </Form.Item>
      )}
    </Card>
  )
}
