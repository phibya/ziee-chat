import { Card, Space, Typography } from 'antd'
import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { useShallow } from 'zustand/react/shallow'
import { useAdminStore, loadSystemProxySettings, updateSystemProxySettings, clearSystemAdminError } from '../../../store'
import { ProxySettingsForm } from './shared'

const { Title, Text } = Typography

export function HttpsProxySettings() {
  const { t } = useTranslation()

  // Admin store
  const {
    proxySettings,
    loading,
    error,
  } = useAdminStore(
    useShallow(state => ({
      proxySettings: state.proxySettings,
      loading: state.loading,
      error: state.error,
    })),
  )

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
      <Space direction="vertical" size="large" style={{ width: '100%' }}>
        <Title level={3}>{t('proxy.title')}</Title>
        <Card>
          <Text type="secondary">{t('proxy.loadingSettings')}</Text>
        </Card>
      </Space>
    )
  }

  return (
    <Space direction="vertical" size="large" style={{ width: '100%' }}>
      <Title level={3}>{t('proxy.title')}</Title>
      <ProxySettingsForm initialSettings={proxySettings} onSave={handleSave} />
    </Space>
  )
}
