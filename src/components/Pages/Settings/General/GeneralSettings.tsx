import { App, Flex, Form } from 'antd'
import { useEffect, useState } from 'react'
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
  const { message } = App.useApp()
  const [form] = Form.useForm()
  const [experimentalFeatures, setExperimentalFeatures] = useState(false)
  const [spellCheck, setSpellCheck] = useState(true)
  const { isDesktop } = Stores.Auth

  useEffect(() => {
    form.setFieldsValue({
      experimentalFeatures,
      spellCheck,
    })
  }, [experimentalFeatures, spellCheck]) // Removed form from dependencies to prevent infinite rerenders

  const handleFormChange = async (changedValues: any) => {
    try {
      if ('experimentalFeatures' in changedValues) {
        setExperimentalFeatures(changedValues.experimentalFeatures)
        message.success(
          changedValues.experimentalFeatures
            ? t('admin.experimentalEnabled')
            : t('admin.experimentalDisabled'),
        )
      }
      if ('spellCheck' in changedValues) {
        setSpellCheck(changedValues.spellCheck)
        message.success(
          changedValues.spellCheck
            ? t('general.spellCheckEnabled')
            : t('general.spellCheckDisabled'),
        )
      }
    } catch (error: any) {
      message.error(error?.message || t('common.failedToUpdate'))
      form.setFieldsValue({
        experimentalFeatures,
        spellCheck,
      })
    }
  }

  return (
    <SettingsPageContainer title={t('pages.general')}>
      <Flex className={'flex-col gap-3 pb-2'}>
        {isDesktop && (
          <>
            <ApplicationCard />
            <AdvancedCard
              form={form}
              experimentalFeatures={experimentalFeatures}
              onFormChange={handleFormChange}
            />
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
