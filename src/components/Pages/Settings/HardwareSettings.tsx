import { Card, Flex, Typography } from 'antd'
import { useTranslation } from 'react-i18next'

const { Title, Text } = Typography

export function HardwareSettings() {
  const { t } = useTranslation()
  return (
    <Flex vertical className="gap-4 w-full">
      <Title level={3}>{t('pages.hardware')}</Title>
      <Card title={t('settings.hardwareConfiguration')}>
        <Text type="secondary">
          {t('settings.hardwareConfigurationDescription')}
        </Text>
      </Card>
    </Flex>
  )
}
