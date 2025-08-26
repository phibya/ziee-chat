import { Card, Flex, Typography } from 'antd'
import { useTranslation } from 'react-i18next'
import { isTauriView } from '../../../../api/core.ts'
import { SettingsPageContainer } from '../common/SettingsPageContainer.tsx'
import { ApplicationCard } from './ApplicationCard'
import { AdvancedCard } from './AdvancedCard'
import { DataFolderCard } from './DataFolderCard'

const { Text } = Typography

export function AdminGeneralSettings() {
  const { t } = useTranslation()

  // Only show these settings for web app (not desktop)
  if (isTauriView) {
    return (
      <SettingsPageContainer title={t('admin.title')}>
        <Card>
          <Text type="secondary">{t('admin.notAvailableDesktop')}</Text>
        </Card>
      </SettingsPageContainer>
    )
  }

  return (
    <SettingsPageContainer title={t('admin.title')}>
      <Flex className={'flex-col gap-3 w-full'}>
        <ApplicationCard isAdmin={true} />
        <AdvancedCard
          isAdmin={true}
        />
        <DataFolderCard isAdmin={true} />
      </Flex>
    </SettingsPageContainer>
  )
}
