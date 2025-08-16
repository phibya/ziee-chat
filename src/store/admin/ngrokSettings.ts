import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../../api/client'
import type { NgrokSettingsResponse, UpdateNgrokSettingsRequest, NgrokStatusResponse, UpdateAccountPasswordRequest } from '../../types/api/config'

interface AdminNgrokSettingsState {
  ngrokSettings: NgrokSettingsResponse | null
  ngrokStatus: NgrokStatusResponse | null
  loadingSettings: boolean
  loadingStatus: boolean
  error: string | null
}

export const useAdminNgrokSettingsStore = create<AdminNgrokSettingsState>()(
  subscribeWithSelector(
    (): AdminNgrokSettingsState => ({
      ngrokSettings: null,
      ngrokStatus: null,
      loadingSettings: false,
      loadingStatus: false,
      error: null,
    }),
  ),
)

// Store methods - defined OUTSIDE the store definition

export const loadNgrokSettings = async (): Promise<void> => {
  useAdminNgrokSettingsStore.setState({ loadingSettings: true, error: null })

  try {
    const settings = await ApiClient.Admin.getNgrokSettings()
    useAdminNgrokSettingsStore.setState({
      ngrokSettings: settings,
      loadingSettings: false,
      error: null,
    })
  } catch (error) {
    console.error('Failed to load ngrok settings:', error)
    useAdminNgrokSettingsStore.setState({
      loadingSettings: false,
      error: error instanceof Error ? error.message : 'Unknown error',
    })
  }
}

export const updateNgrokSettings = async (settings: UpdateNgrokSettingsRequest): Promise<void> => {
  useAdminNgrokSettingsStore.setState({ loadingSettings: true, error: null })

  try {
    const updatedSettings = await ApiClient.Admin.updateNgrokSettings(settings)
    useAdminNgrokSettingsStore.setState({
      ngrokSettings: updatedSettings,
      loadingSettings: false,
      error: null,
    })
  } catch (error) {
    console.error('Failed to update ngrok settings:', error)
    useAdminNgrokSettingsStore.setState({
      loadingSettings: false,
      error: error instanceof Error ? error.message : 'Unknown error',
    })
    throw error
  }
}

export const updateAccountPassword = async (passwordData: UpdateAccountPasswordRequest): Promise<void> => {
  try {
    await ApiClient.User.updateAccountPassword(passwordData)
  } catch (error) {
    console.error('Failed to update account password:', error)
    throw error
  }
}

export const startNgrokTunnel = async (): Promise<void> => {
  useAdminNgrokSettingsStore.setState({ loadingStatus: true, error: null })

  try {
    const status = await ApiClient.Admin.startNgrokTunnel()
    useAdminNgrokSettingsStore.setState({
      ngrokStatus: status,
      loadingStatus: false,
      error: null,
    })
    // Refresh settings to get updated tunnel URL
    await loadNgrokSettings()
  } catch (error) {
    console.error('Failed to start ngrok tunnel:', error)
    useAdminNgrokSettingsStore.setState({
      loadingStatus: false,
      error: error instanceof Error ? error.message : 'Unknown error',
    })
    throw error
  }
}

export const stopNgrokTunnel = async (): Promise<void> => {
  useAdminNgrokSettingsStore.setState({ loadingStatus: true, error: null })

  try {
    const status = await ApiClient.Admin.stopNgrokTunnel()
    useAdminNgrokSettingsStore.setState({
      ngrokStatus: status,
      loadingStatus: false,
      error: null,
    })
    // Refresh settings
    await loadNgrokSettings()
  } catch (error) {
    console.error('Failed to stop ngrok tunnel:', error)
    useAdminNgrokSettingsStore.setState({
      loadingStatus: false,
      error: error instanceof Error ? error.message : 'Unknown error',
    })
    throw error
  }
}

export const refreshNgrokStatus = async (): Promise<void> => {
  try {
    const status = await ApiClient.Admin.getNgrokStatus()
    useAdminNgrokSettingsStore.setState({
      ngrokStatus: status,
      error: null,
    })
  } catch (error) {
    console.error('Failed to refresh ngrok status:', error)
    useAdminNgrokSettingsStore.setState({
      error: error instanceof Error ? error.message : 'Unknown error',
    })
  }
}