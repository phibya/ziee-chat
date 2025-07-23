import { Card, Flex, Typography } from 'antd'
import { useTranslation } from 'react-i18next'

const { Title, Text } = Typography

export function LocalApiServerSettings() {
  const { t } = useTranslation()
  return (
    <Flex vertical className="gap-4 w-full">
      <Title level={3}>Local API Server</Title>
      <Card title={t('settings.serverConfiguration')}>
        <Text type="secondary">
          Local API server configuration will be implemented here.
        </Text>
      </Card>
    </Flex>
  )
}
