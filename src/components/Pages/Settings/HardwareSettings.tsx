import { Card, Space, Typography } from 'antd'
import { useTranslation } from 'react-i18next'

const { Title, Text } = Typography

export function HardwareSettings() {
  const { t } = useTranslation()
  return (
    <Space direction="vertical" size="large" style={{ width: '100%' }}>
      <Title level={3}>{t('pages.hardware')}</Title>
      <Card title={t('settings.hardwareConfiguration')}>
        <Text type="secondary">
          {t('settings.hardwareConfigurationDescription')}
        </Text>
      </Card>
    </Space>
  )
}
