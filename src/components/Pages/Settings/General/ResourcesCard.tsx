import { Button, Card, Divider, Flex, Typography } from 'antd'
import { useTranslation } from 'react-i18next'

const { Text } = Typography

export function ResourcesCard() {
  const { t } = useTranslation()

  return (
    <Card title={t('general.resources')}>
      <Flex vertical className="gap-2 w-full">
        <Flex
          justify="space-between"
          align="flex-start"
          wrap
          gap="small"
          className="min-w-0"
        >
          <div className="flex-1 min-w-80">
            <Text strong>{t('general.documentation')}</Text>
            <div>
              <Text type="secondary">
                {t('general.documentationDescription')}
              </Text>
            </div>
          </div>
          <div className="flex-shrink-0">
            <Button type="link">{t('buttons.viewDocs')}</Button>
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
            <Text strong>{t('general.releaseNotes')}</Text>
            <div>
              <Text type="secondary">
                {t('general.releaseNotesDescription')}
              </Text>
            </div>
          </div>
          <div className="flex-shrink-0">
            <Button type="link">{t('buttons.viewReleases')}</Button>
          </div>
        </Flex>
      </Flex>
    </Card>
  )
}
