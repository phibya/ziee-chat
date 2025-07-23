import { Card, Flex, Typography } from 'antd'
import { useTranslation } from 'react-i18next'

const { Title, Text } = Typography

export function ShortcutsSettings() {
  const { t } = useTranslation()
  return (
    <Flex vertical className="gap-4 w-full">
      <Title level={3}>{t('pages.shortcuts')}</Title>
      <Card title={t('settings.keyboardShortcuts')}>
        <Text type="secondary">
          {t('settings.keyboardShortcutsDescription')}
        </Text>
      </Card>
    </Flex>
  )
}
