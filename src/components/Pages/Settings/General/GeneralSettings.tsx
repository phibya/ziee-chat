import { Flex } from 'antd'
import { useTranslation } from 'react-i18next'
import { SettingsPageContainer } from '../common/SettingsPageContainer.tsx'
import { Stores } from '../../../../store'
import { ApplicationCard } from './ApplicationCard'
import { AdvancedCard } from './AdvancedCard'
import { DataFolderCard } from './DataFolderCard'
import { ResourcesCard } from './ResourcesCard'
import { CommunityCard } from './CommunityCard'
import { SupportCard } from './SupportCard'

export function GeneralSettings() {
  const { t } = useTranslation()
  const { isDesktop } = Stores.Auth

  return (
    <SettingsPageContainer title={t('pages.general')}>
      <Flex className={'flex-col gap-3 pb-2'}>
        {isDesktop && (
          <>
            <ApplicationCard />
            <AdvancedCard />
            <DataFolderCard />
          </>
        )}

        <ResourcesCard />
        <CommunityCard />
        <SupportCard />
      </Flex>
    </SettingsPageContainer>
  )
}
