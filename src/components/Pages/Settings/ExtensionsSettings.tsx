import { Card, Flex, Typography } from 'antd'
import { useTranslation } from 'react-i18next'

const { Title, Text } = Typography

export function ExtensionsSettings() {
  const { t } = useTranslation()
  return (
    <Flex vertical className="gap-4 w-full">
      <Title level={3}>Extensions</Title>
      <Card title={t('settings.extensionManagement')}>
        <Text type="secondary">
          Extensions management will be implemented here.
        </Text>
      </Card>
    </Flex>
  )
}
