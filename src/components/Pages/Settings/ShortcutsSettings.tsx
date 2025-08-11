import { Card, Typography } from 'antd'
import { useTranslation } from 'react-i18next'
import { SettingsPageContainer } from './common/SettingsPageContainer.tsx'

const { Text } = Typography

export function ShortcutsSettings() {
  const { t } = useTranslation()
  return (
    <SettingsPageContainer title={t('pages.shortcuts')}>
      <Card title={t('settings.keyboardShortcuts')}>
        <Text type="secondary">
          {t('settings.keyboardShortcutsDescription')}
        </Text>
      </Card>
    </SettingsPageContainer>
  )
}
