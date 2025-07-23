import { Card, Flex, Typography } from 'antd'
import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import {
  clearSystemAdminError,
  loadSystemProxySettings,
  Stores,
  updateSystemProxySettings,
} from '../../../store'
import { ProxySettingsForm } from './shared'

const { Title, Text } = Typography

export function HttpsProxySettings() {
  const { t } = useTranslation()

  // Admin store
  const { proxySettings, loading, error } = Stores.Admin

  useEffect(() => {
    loadSystemProxySettings()
  }, [])

  // Show errors from store
  useEffect(() => {
    if (error) {
      clearSystemAdminError()
    }
  }, [error])

  const handleSave = async (values: any) => {
    await updateSystemProxySettings(values)
  }

  if (loading && !proxySettings) {
    return (
      <Flex vertical className="gap-4 w-full">
        <Title level={3}>{t('proxy.title')}</Title>
        <Card>
          <Text type="secondary">{t('proxy.loadingSettings')}</Text>
        </Card>
      </Flex>
    )
  }

  return (
    <Flex vertical className="gap-4 w-full">
      <Title level={3}>{t('proxy.title')}</Title>
      <ProxySettingsForm initialSettings={proxySettings} onSave={handleSave} />
    </Flex>
  )
}
