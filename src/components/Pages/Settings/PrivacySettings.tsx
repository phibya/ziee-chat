import { Card, Space, Typography } from 'antd'
import { useTranslation } from 'react-i18next'

const { Title, Text } = Typography

export function PrivacySettings() {
  const { t } = useTranslation()
  return (
    <Space direction="vertical" size="large" style={{ width: '100%' }}>
      <Title level={3}>{t('pages.privacy')}</Title>
      <Card title={t('settings.privacyControls')}>
        <Text type="secondary">{t('settings.privacySettingsDescription')}</Text>
      </Card>
    </Space>
  )
}
