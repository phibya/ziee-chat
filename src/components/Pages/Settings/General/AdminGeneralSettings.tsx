import { App, Card, Flex, Form, Typography } from 'antd'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { isTauriView } from '../../../../api/core.ts'
import { SettingsPageContainer } from '../common/SettingsPageContainer.tsx'
import { ApplicationCard } from './ApplicationCard'
import { AdvancedCard } from './AdvancedCard'
import { DataFolderCard } from './DataFolderCard'

const { Text } = Typography

export function AdminGeneralSettings() {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const [form] = Form.useForm()
  const [experimentalFeatures, setExperimentalFeatures] = useState(false)

  useEffect(() => {
    form.setFieldsValue({
      experimentalFeatures,
    })
  }, [experimentalFeatures]) // Removed form from dependencies to prevent infinite rerenders

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
    } catch (error: any) {
      message.error(error?.message || t('common.failedToUpdate'))
      form.setFieldsValue({
        experimentalFeatures,
      })
    }
  }

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
          form={form}
          experimentalFeatures={experimentalFeatures}
          onFormChange={handleFormChange}
          isAdmin={true}
        />
        <DataFolderCard isAdmin={true} />
      </Flex>
    </SettingsPageContainer>
  )
}
