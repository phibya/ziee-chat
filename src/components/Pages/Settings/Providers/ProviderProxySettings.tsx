import { ProxySettings } from '../../../../types'
import { ProxySettingsForm } from '../common'

interface ProviderProxySettingsProps {
  initialSettings: ProxySettings
  onSave: (settings: ProxySettings) => void
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
