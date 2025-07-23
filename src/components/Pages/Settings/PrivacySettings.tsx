import { Card, Flex, Typography } from 'antd'
import { useTranslation } from 'react-i18next'

const { Title, Text } = Typography

export function PrivacySettings() {
  const { t } = useTranslation()
  return (
    <Flex vertical className="gap-4 w-full">
      <Title level={3}>{t('pages.privacy')}</Title>
      <Card title={t('settings.privacyControls')}>
        <Text type="secondary">{t('settings.privacySettingsDescription')}</Text>
      </Card>
    </Flex>
  )
}
