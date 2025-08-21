import { Button, Card, Divider, Flex, Typography } from 'antd'
import { FileTextOutlined, FolderOpenOutlined } from '@ant-design/icons'
import { useTranslation } from 'react-i18next'

const { Text } = Typography

interface DataFolderCardProps {
  isAdmin?: boolean
}

export function DataFolderCard({ isAdmin = false }: DataFolderCardProps) {
  const { t } = useTranslation()

  const dataPath = isAdmin
    ? '/var/lib/app/data'
    : '/Users/user/Library/Application Support/Ziee/data'

  return (
    <Card title={isAdmin ? t('admin.dataFolder') : t('general.dataFolder')}>
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
              {isAdmin ? t('admin.appData') : t('labels.appData')}
            </Text>
            <div>
              <Text type="secondary">
                {isAdmin
                  ? t('admin.appDataDesc')
                  : t('general.appDataDescription')}
              </Text>
            </div>
            <div>
              <Text type="secondary" style={{ fontSize: '12px' }}>
                {dataPath}
              </Text>
            </div>
          </div>
          <div className="flex-shrink-0">
            <Button type="default" icon={<FolderOpenOutlined />}>
              {isAdmin
                ? t('admin.changeLocation')
                : t('buttons.changeLocation')}
            </Button>
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
              {isAdmin ? t('admin.appLogs') : t('general.appLogs')}
            </Text>
            <div>
              <Text type="secondary">
                {isAdmin
                  ? t('admin.appLogsDesc')
                  : t('general.appLogsDescription')}
              </Text>
            </div>
          </div>
          <Flex wrap gap="small" className="flex-shrink-0">
            <Button type="default" icon={<FileTextOutlined />}>
              {isAdmin ? t('admin.openLogs') : t('buttons.openLogs')}
            </Button>
            <Button type="default" icon={<FolderOpenOutlined />}>
              {isAdmin ? t('admin.showInFinder') : t('buttons.showInFinder')}
            </Button>
          </Flex>
        </Flex>
      </Flex>
    </Card>
  )
}
