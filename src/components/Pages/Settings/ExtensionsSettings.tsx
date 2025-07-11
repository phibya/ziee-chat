import { Card, Space, Typography } from 'antd'
import { useTranslation } from 'react-i18next'

const { Title, Text } = Typography

export function ExtensionsSettings() {
  const { t } = useTranslation()
  return (
    <Space direction="vertical" size="large" style={{ width: '100%' }}>
      <Title level={3}>Extensions</Title>
      <Card title={t('settings.extensionManagement')}>
        <Text type="secondary">
          Extensions management will be implemented here.
        </Text>
      </Card>
    </Space>
  )
}
