import { Button, Card, Divider, Flex, Typography } from 'antd'
import { useTranslation } from 'react-i18next'

const { Text } = Typography

interface ApplicationCardProps {
  isAdmin?: boolean
}

export function ApplicationCard({ isAdmin = false }: ApplicationCardProps) {
  const { t } = useTranslation()

  return (
    <Card title={isAdmin ? t('admin.application') : t('general.application')}>
      <Flex vertical className="gap-2 w-full">
        <Flex
          justify="space-between"
          align="flex-start"
          wrap
          gap="small"
          className="min-w-0"
        >
          <div className="flex-1 min-w-80">
            <Text strong>
              {isAdmin ? t('admin.appVersion') : t('labels.appVersion')}
            </Text>
            <div>
              <Text type="secondary">v0.6.4</Text>
            </div>
          </div>
          <div className="flex-shrink-0">
            <Text type="secondary">v0.6.4</Text>
          </div>
        </Flex>
        <Divider style={{ margin: 0 }} />
        <Flex
          justify="space-between"
          align="flex-start"
          wrap
          gap="small"
          className="min-w-0"
        >
          <div className="flex-1 min-w-80">
            <Text strong>
              {isAdmin
                ? t('admin.checkForUpdates')
                : t('labels.checkForUpdates')}
            </Text>
            <div>
              <Text type="secondary">
                {isAdmin
                  ? t('admin.checkForUpdatesDesc')
                  : t('general.checkForUpdatesDescription')}
              </Text>
            </div>
          </div>
          <div className="flex-shrink-0">
            <Button type="default">
              {isAdmin
                ? t('admin.checkForUpdates')
                : t('buttons.checkForUpdates')}
            </Button>
          </div>
        </Flex>
      </Flex>
    </Card>
  )
}
