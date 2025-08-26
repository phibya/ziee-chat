import { App, Card, Divider, Flex, Form } from 'antd'
import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { useParams } from 'react-router-dom'
import {
  clearProvidersError,
  Stores,
} from '../../../../store'
import { DownloadInstance } from '../../../../types'
import { DownloadItem } from '../../../common/DownloadItem'
import { ModelsSection } from './common/ModelsSection'
import { ProviderHeader } from './common/ProviderHeader'

export function LocalProviderSettings() {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const { providerId } = useParams<{ providerId?: string }>()

  const [nameForm] = Form.useForm()

  // Store data
  const { error } = Stores.AdminProviders
  const { downloads } = Stores.ModelDownload

  // Get current provider
  const currentProvider = Stores.AdminProviders.providers.find(
    p => p.id === providerId,
  )

  // Get active downloads for this provider
  const providerDownloads = downloads.filter(
    (download: DownloadInstance) => download.provider_id === providerId,
  )


  // Show errors
  useEffect(() => {
    if (error) {
      message.error(error)
      clearProvidersError()
    }
  }, [error]) // Removed message from dependencies to prevent infinite rerenders

  // Update form when provider changes
  useEffect(() => {
    if (currentProvider) {
      nameForm.setFieldsValue({
        name: currentProvider.name,
      })
    }
  }, [currentProvider, nameForm])

  // Return early if no provider or not local
  if (!currentProvider || currentProvider.type !== 'local') {
    return null
  }

  return (
    <Flex className={'flex-col gap-3'}>
      <ProviderHeader />

      {/* Downloads Section - For Local providers only */}
      {providerDownloads.length > 0 && (
        <Card title={t('providers.downloadingModels')}>
          <Flex vertical>
            {providerDownloads.map((download: DownloadInstance, i: number) => (
              <>
                <DownloadItem key={download.id} download={download} />
                {i < providerDownloads.length - 1 && (
                  <Divider className={'m-0'} />
                )}
              </>
            ))}
          </Flex>
        </Card>
      )}

      {/* Models Section */}
      <ModelsSection />
    </Flex>
  )
}
