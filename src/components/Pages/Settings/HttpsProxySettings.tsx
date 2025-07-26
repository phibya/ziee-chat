import { Card, Typography } from 'antd'
import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import {
  clearSystemAdminError,
  loadSystemProxySettings,
  Stores,
  updateSystemProxySettings,
} from '../../../store'
import { ProxySettingsForm } from './shared'
import { SettingsPageContainer } from './SettingsPageContainer'

const { Text } = Typography

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
      <SettingsPageContainer title={t('proxy.title')}>
        <Card>
          <Text type="secondary">{t('proxy.loadingSettings')}</Text>
        </Card>
      </SettingsPageContainer>
    )
  }

  return (
    <SettingsPageContainer title={t('proxy.title')}>
      <ProxySettingsForm initialSettings={proxySettings} onSave={handleSave} />
    </SettingsPageContainer>
  )
}
