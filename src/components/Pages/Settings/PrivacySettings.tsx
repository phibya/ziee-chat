import { Card, Typography } from 'antd'
import { useTranslation } from 'react-i18next'
import { SettingsPageContainer } from './common/SettingsPageContainer.tsx'

const { Text } = Typography

export function PrivacySettings() {
  const { t } = useTranslation()
  return (
    <SettingsPageContainer title={t('pages.privacy')}>
      <Card title={t('settings.privacyControls')}>
        <Text type="secondary">{t('settings.privacySettingsDescription')}</Text>
      </Card>
    </SettingsPageContainer>
  )
}
