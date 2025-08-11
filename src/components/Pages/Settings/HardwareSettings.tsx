import { Card, Typography } from 'antd'
import { useTranslation } from 'react-i18next'
import { SettingsPageContainer } from './common/SettingsPageContainer.tsx'

const { Text } = Typography

export function HardwareSettings() {
  const { t } = useTranslation()
  return (
    <SettingsPageContainer title={t('pages.hardware')}>
      <Card title={t('settings.hardwareConfiguration')}>
        <Text type="secondary">
          {t('settings.hardwareConfigurationDescription')}
        </Text>
      </Card>
    </SettingsPageContainer>
  )
}
