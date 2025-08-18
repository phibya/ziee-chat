import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../../api/client'
import { ProxySettings, UpdateProxySettingsRequest } from '../../types'

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
  settings: UpdateProxySettingsRequest,
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
