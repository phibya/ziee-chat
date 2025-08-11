import { Card, Typography } from 'antd'
import { useTranslation } from 'react-i18next'
import { SettingsPageContainer } from './common/SettingsPageContainer.tsx'

const { Text } = Typography

export function ExtensionsSettings() {
  const { t } = useTranslation()
  return (
    <SettingsPageContainer title="Extensions">
      <Card title={t('settings.extensionManagement')}>
        <Text type="secondary">
          Extensions management will be implemented here.
        </Text>
      </Card>
    </SettingsPageContainer>
  )
}
