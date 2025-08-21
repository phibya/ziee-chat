import { Button, Card, Divider, Flex, Typography } from 'antd'
import { useTranslation } from 'react-i18next'
import { IoLogoGithub } from 'react-icons/io'
import { FaDiscord } from 'react-icons/fa'

const { Text } = Typography

export function CommunityCard() {
  const { t } = useTranslation()

  return (
    <Card title={t('general.community')}>
      <Flex vertical className="gap-2 w-full">
        <Flex
          justify="space-between"
          align="flex-start"
          wrap
          gap="small"
          className="min-w-0"
        >
          <div className="flex-1 min-w-80">
            <Text strong>{t('general.github')}</Text>
            <div>
              <Text type="secondary">{t('general.githubDescription')}</Text>
            </div>
          </div>
          <div className="flex-shrink-0">
            <Button type="text">
              <IoLogoGithub />
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
            <Text strong>{t('general.discord')}</Text>
            <div>
              <Text type="secondary">{t('general.discordDescription')}</Text>
            </div>
          </div>
          <div className="flex-shrink-0">
            <Button type="text">
              <FaDiscord />
            </Button>
          </div>
        </Flex>
      </Flex>
    </Card>
  )
}
