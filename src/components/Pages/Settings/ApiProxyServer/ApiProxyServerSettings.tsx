import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { Card, Typography, Alert } from 'antd'
import { SettingsPageContainer } from '../common/SettingsPageContainer.tsx'
import { Permission, usePermissions } from '../../../../permissions'
import { Stores } from '../../../../store'
import { initializeApiProxyServerData } from '../../../../store/admin/apiProxyServer.ts'
import { ServerConfigurationCard } from './ServerConfigurationCard'
import { ModelSelectionCard } from './ModelSelectionCard'
import { TrustedHostsCard } from './TrustedHostsCard'
import { ServerControlCard } from './ServerControlCard'

const { Text } = Typography

export function ApiProxyServerSettings() {
  const { t } = useTranslation()
  const { hasPermission } = usePermissions()

  // Permission check
  const canRead = hasPermission(Permission.config.apiProxyServer?.read)

  if (!canRead) {
    return (
      <SettingsPageContainer title={t('apiProxyServer.title')}>
        <Card>
          <Text type="secondary">{t('permissions.insufficient')}</Text>
        </Card>
      </SettingsPageContainer>
    )
  }

  // Store data
  const { error, initialized } = Stores.AdminApiProxyServer

  // Load data on mount
  useEffect(() => {
    if (!initialized) {
      initializeApiProxyServerData().catch(console.error)
    }
  }, [initialized])

  return (
    <SettingsPageContainer
      title={t('apiProxyServer.title')}
      subtitle={t('apiProxyServer.subtitle')}
    >
      <div className="flex flex-col gap-3 flex-wrap w-full">
        {error && (
          <Alert
            message={t('apiProxyServer.error')}
            description={error}
            type="error"
            showIcon
            closable
          />
        )}

        {/* Configuration Card */}
        <ServerConfigurationCard />

        {/* Model Selection Card */}
        <ModelSelectionCard />

        {/* Trusted Hosts Card */}
        <TrustedHostsCard />

        {/* Server Control Card */}
        <ServerControlCard />
      </div>
    </SettingsPageContainer>
  )
}
