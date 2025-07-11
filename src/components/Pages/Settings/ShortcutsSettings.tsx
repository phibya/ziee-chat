import { Card, Space, Typography } from 'antd'
import { useTranslation } from 'react-i18next'

const { Title, Text } = Typography

export function ShortcutsSettings() {
  const { t } = useTranslation()
  return (
    <Space direction="vertical" size="large" style={{ width: '100%' }}>
      <Title level={3}>{t('pages.shortcuts')}</Title>
      <Card title={t('settings.keyboardShortcuts')}>
        <Text type="secondary">
          {t('settings.keyboardShortcutsDescription')}
        </Text>
      </Card>
    </Space>
  )
}
