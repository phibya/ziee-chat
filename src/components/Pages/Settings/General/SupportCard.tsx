import { Button, Card, Flex, Typography } from 'antd'
import { useTranslation } from 'react-i18next'

const { Text } = Typography

export function SupportCard() {
  const { t } = useTranslation()

  return (
    <Card title={t('general.support')}>
      <Flex justify="space-between" align="center">
        <div>
          <Text strong>{t('general.reportAnIssue')}</Text>
          <div>
            <Text type="secondary">
              {t('general.reportAnIssueDescription')}
            </Text>
          </div>
        </div>
        <Button type="link">{t('buttons.reportIssue')}</Button>
      </Flex>
    </Card>
  )
}
