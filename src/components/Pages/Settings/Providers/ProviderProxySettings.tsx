import { ProviderProxySettings } from '../../../../types/api/provider'
import { ProxySettingsForm } from '../shared'

interface ProviderProxySettingsProps {
  initialSettings: ProviderProxySettings
  onSave: (settings: ProviderProxySettings) => void
  disabled?: boolean
}

export function ProviderProxySettingsForm({
  initialSettings,
  onSave,
  disabled = false,
}: ProviderProxySettingsProps) {

  return (
    <ProxySettingsForm
      initialSettings={initialSettings}
      onSave={onSave}
      disabled={disabled}
    />
  )
}
