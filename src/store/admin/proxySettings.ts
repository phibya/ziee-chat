import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../../api/client'
import { UpdateProxySettingsRequest } from '../../types/api/config'

type ProxySettings = UpdateProxySettingsRequest

interface AdminProxySettingsState {
  // Data
  proxySettings: ProxySettings | null

  // Loading states
  loading: boolean
  loadingProxySettings: boolean
  updating: boolean

  // Error state
  error: string | null
}

export const useAdminProxySettingsStore = create<AdminProxySettingsState>()(
  subscribeWithSelector(
    (): AdminProxySettingsState => ({
      // Initial state
      proxySettings: null,
      loading: false,
      loadingProxySettings: false,
      updating: false,
      error: null,
    }),
  ),
)

// Proxy settings actions
export const loadSystemProxySettings = async (): Promise<void> => {
  try {
    useAdminProxySettingsStore.setState({
      loadingProxySettings: true,
      error: null,
    })

    const settings = await ApiClient.Admin.getProxySettings()

    useAdminProxySettingsStore.setState({
      proxySettings: {
        enabled: settings.enabled,
        url: settings.url,
        username: settings.username,
        password: settings.password,
        no_proxy: settings.no_proxy,
        ignore_ssl_certificates: settings.ignore_ssl_certificates,
        proxy_ssl: settings.proxy_ssl,
        proxy_host_ssl: settings.proxy_host_ssl,
        peer_ssl: settings.peer_ssl,
        host_ssl: settings.host_ssl,
      },
      loadingProxySettings: false,
    })
  } catch (error) {
    useAdminProxySettingsStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to load proxy settings',
      loadingProxySettings: false,
    })
    throw error
  }
}

export const updateSystemProxySettings = async (
  settings: ProxySettings,
): Promise<void> => {
  try {
    useAdminProxySettingsStore.setState({ updating: true, error: null })

    await ApiClient.Admin.updateProxySettings(settings)

    useAdminProxySettingsStore.setState({
      proxySettings: settings,
      updating: false,
    })
  } catch (error) {
    useAdminProxySettingsStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to update proxy settings',
      updating: false,
    })
    throw error
  }
}

export const clearAdminProxySettingsStoreError = (): void => {
  useAdminProxySettingsStore.setState({ error: null })
}
