import { Card, Space, Typography } from 'antd'
import { useTranslation } from 'react-i18next'

const { Title, Text } = Typography

export function LocalApiServerSettings() {
  const { t } = useTranslation()
  return (
    <Space direction="vertical" size="large" style={{ width: '100%' }}>
      <Title level={3}>Local API Server</Title>
      <Card title={t('settings.serverConfiguration')}>
        <Text type="secondary">
          Local API server configuration will be implemented here.
        </Text>
      </Card>
    </Space>
  )
}
