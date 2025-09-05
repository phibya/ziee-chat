import { useTranslation } from 'react-i18next'
import { SettingsPageContainer } from '../common/SettingsPageContainer.tsx'
import { ServerConfigurationCard } from './ServerConfigurationCard'
import { ModelSelectionCard } from './ModelSelectionCard'
import { TrustedHostsCard } from './TrustedHostsCard'
import { ServerControlCard } from './ServerControlCard'

export function ApiProxyServerSettings() {
  const { t } = useTranslation()

  // Store data

  return (
    <SettingsPageContainer
      title={t('apiProxyServer.title')}
      subtitle={t('apiProxyServer.subtitle')}
    >
      {/* Configuration Card */}
      <ServerConfigurationCard />

      {/* Model Selection Card */}
      <ModelSelectionCard />

      {/* Trusted Hosts Card */}
      <TrustedHostsCard />

      {/* Server Control Card */}
      <ServerControlCard />
    </SettingsPageContainer>
  )
}
