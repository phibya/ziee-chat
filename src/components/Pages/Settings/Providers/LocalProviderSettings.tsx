import { CloseOutlined, PlusOutlined, UploadOutlined } from '@ant-design/icons'
import {
  App,
  Button,
  Card,
  Dropdown,
  Flex,
  Form,
  List,
  Progress,
  Typography,
} from 'antd'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useParams } from 'react-router-dom'
import { isDesktopApp } from '../../../../api/core'
import { Permission, usePermissions } from '../../../../permissions'
import {
  clearProvidersError,
  deleteExistingModel,
  deleteModelDownload,
  disableModelFromUse,
  enableModelForUse,
  loadModels,
  openAddLocalModelDownloadModal,
  openAddLocalModelUploadModal,
  openEditLocalModelModal,
  openViewDownloadModal,
  startModelExecution,
  stopModelExecution,
  Stores,
  updateModelProvider,
} from '../../../../store'
import { DownloadInstance, Provider } from '../../../../types'
import { ModelsSection } from './shared/ModelsSection'
import { ProviderHeader } from './shared/ProviderHeader'

const { Text } = Typography

export function LocalProviderSettings() {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const { hasPermission } = usePermissions()
  const { provider_id } = useParams<{ provider_id?: string }>()

  const [nameForm] = Form.useForm()
  const [isMobile, setIsMobile] = useState(false)

  // Store data
  const { providers, modelsByProvider, loadingModels, modelOperations, error } =
    Stores.Providers
  const { downloads } = Stores.ModelDownload

  // Check permissions for web app
  const canEditProviders =
    isDesktopApp || hasPermission(Permission.config.providers.edit)

  // Find current provider
  const currentProvider = providers.find(p => p.id === provider_id)
  const currentModels = provider_id ? modelsByProvider[provider_id] || [] : []
  const modelsLoading = provider_id
    ? loadingModels[provider_id] || false
    : false

  // Get active downloads for this provider
  const providerDownloads = Object.values(downloads).filter(
    (download: DownloadInstance) => download.provider_id === provider_id,
  )

  // Format bytes to human readable format
  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`
  }

  const formatSpeed = (speedBps?: number): string => {
    if (!speedBps) return ''

    const speedMBps = speedBps / (1024 * 1024)
    if (speedMBps >= 1) {
      return `${speedMBps.toFixed(1)} MB/s`
    }

    const speedKBps = speedBps / 1024
    return `${speedKBps.toFixed(1)} KB/s`
  }

  const formatETA = (etaSeconds?: number): string => {
    if (!etaSeconds) return ''

    const hours = Math.floor(etaSeconds / 3600)
    const minutes = Math.floor((etaSeconds % 3600) / 60)
    const seconds = Math.floor(etaSeconds % 60)

    if (hours > 0) {
      return `${hours}h ${minutes}m`
    } else if (minutes > 0) {
      return `${minutes}m ${seconds}s`
    } else {
      return `${seconds}s`
    }
  }

  // Helper functions for provider validation
  const canEnableProvider = (provider: Provider): boolean => {
    if (provider.enabled) return true // Already enabled
    const providerModels = modelsByProvider[provider.id] || []
    if (providerModels.length === 0) return false
    if (provider.type === 'local') return true
    if (!provider.api_key || provider.api_key.trim() === '') return false
    if (!provider.base_url || provider.base_url.trim() === '') return false
    try {
      new globalThis.URL(provider.base_url)
      return true
    } catch {
      return false
    }
  }

  const getEnableDisabledReason = (provider: Provider): string | null => {
    if (provider.enabled) return null
    const providerModels = modelsByProvider[provider.id] || []
    if (providerModels.length === 0)
      return 'No models available. Add at least one model first.'
    if (provider.type === 'local') return null
    if (!provider.api_key || provider.api_key.trim() === '')
      return 'API key is required'
    if (!provider.base_url || provider.base_url.trim() === '')
      return 'Base URL is required'
    try {
      new globalThis.URL(provider.base_url)
      return null
    } catch {
      return 'Invalid base URL format'
    }
  }

  // Event handlers
  const handleNameChange = async (changedValues: any) => {
    if (!currentProvider || !canEditProviders) return

    try {
      await updateModelProvider(currentProvider.id, {
        name: changedValues.name,
      })
    } catch (error) {
      console.error('Failed to update provider:', error)
      // Error is handled by the store
    }
  }

  const handleProviderToggle = async (providerId: string, enabled: boolean) => {
    if (!canEditProviders) {
      message.error(t('providers.noPermissionModify'))
      return
    }

    try {
      await updateModelProvider(providerId, {
        enabled: enabled,
      })
      const provider = providers.find(p => p.id === providerId)
      message.success(
        `${provider?.name || 'Provider'} ${enabled ? 'enabled' : 'disabled'}`,
      )
    } catch (error: any) {
      console.error('Failed to update provider:', error)
      // Handle error similar to original implementation
      if (error.response?.status === 400) {
        const provider = providers.find(p => p.id === providerId)
        if (provider) {
          const providerModels = modelsByProvider[provider.id] || []
          if (providerModels.length === 0) {
            message.error(
              `Cannot enable "${provider.name}" - No models available`,
            )
          } else {
            message.error(
              `Cannot enable "${provider.name}" - Invalid configuration`,
            )
          }
        } else {
          message.error(error?.message || 'Failed to update provider')
        }
      } else {
        message.error(error?.message || 'Failed to update provider')
      }
    }
  }

  const handleToggleModel = async (modelId: string, enabled: boolean) => {
    if (!currentProvider) return

    try {
      if (enabled) {
        await enableModelForUse(modelId)
      } else {
        await disableModelFromUse(modelId)
      }

      // Check if this was the last enabled model being disabled
      if (!enabled) {
        const providerModels = currentModels
        const remainingEnabledModels = providerModels.filter(
          m => m.id !== modelId && m.enabled !== false,
        )

        // If no models remain enabled and provider is currently enabled, disable the provider
        if (remainingEnabledModels.length === 0 && currentProvider.enabled) {
          try {
            await updateModelProvider(currentProvider.id, { enabled: false })
            const modelName =
              providerModels.find(m => m.id === modelId)?.name || 'Model'
            message.success(
              `${modelName} disabled. ${currentProvider.name} provider disabled as no models remain active.`,
            )
          } catch (providerError) {
            console.error('Failed to disable provider:', providerError)
            const modelName =
              providerModels.find(m => m.id === modelId)?.name || 'Model'
            message.warning(
              `${modelName} disabled, but failed to disable provider automatically`,
            )
          }
        } else {
          const modelName =
            currentModels.find(m => m.id === modelId)?.name || 'Model'
          message.success(`${modelName} ${enabled ? 'enabled' : 'disabled'}`)
        }
      } else {
        const modelName =
          currentModels.find(m => m.id === modelId)?.name || 'Model'
        message.success(`${modelName} ${enabled ? 'enabled' : 'disabled'}`)
      }
    } catch (error) {
      console.error('Failed to toggle model:', error)
      // Error is handled by the store
    }
  }

  const handleDeleteModel = async (modelId: string) => {
    if (!currentProvider) return

    try {
      await deleteExistingModel(modelId)
      message.success(t('providers.modelDeleted'))
    } catch (error) {
      console.error('Failed to delete model:', error)
      // Error is handled by the store
    }
  }

  const handleStartStopModel = async (modelId: string, is_active: boolean) => {
    if (!currentProvider || currentProvider.type !== 'local') return

    try {
      if (is_active) {
        await startModelExecution(modelId)
      } else {
        await stopModelExecution(modelId)
      }

      const modelName =
        currentModels.find(m => m.id === modelId)?.name || 'Model'
      message.success(`${modelName} ${is_active ? 'started' : 'stopped'}`)
    } catch (error) {
      console.error('Failed to start/stop model:', error)
      // Error is handled by the store
    }
  }

  // Effects
  useEffect(() => {
    const checkMobile = () => {
      setIsMobile(window.innerWidth < 768)
    }

    checkMobile()
    window.addEventListener('resize', checkMobile)

    return () => window.removeEventListener('resize', checkMobile)
  }, [])

  // Load models when provider is selected
  useEffect(() => {
    if (
      provider_id &&
      !modelsByProvider[provider_id] &&
      !loadingModels[provider_id]
    ) {
      loadModels(provider_id)
    }
  }, [
    provider_id,
    provider_id ? modelsByProvider[provider_id] : undefined,
    provider_id ? loadingModels[provider_id] : undefined,
  ])

  // Show errors
  useEffect(() => {
    if (error) {
      message.error(error)
      clearProvidersError()
    }
  }, [error]) // Removed message from dependencies to prevent infinite rerenders

  // Update form when provider changes
  useEffect(() => {
    if (currentProvider) {
      nameForm.setFieldsValue({
        name: currentProvider.name,
      })
    }
  }, [currentProvider]) // Removed nameForm from dependencies to prevent infinite rerenders

  // Return early if no provider or not local
  if (!currentProvider || currentProvider.type !== 'local') {
    return null
  }

  const addModelButton = (
    <Dropdown
      menu={{
        items: [
          {
            key: 'upload',
            label: 'Upload from Files',
            icon: <UploadOutlined />,
            onClick: () => openAddLocalModelUploadModal(currentProvider.id),
          },
          {
            key: 'download',
            label: 'Download from Repository',
            icon: <PlusOutlined />,
            onClick: () => openAddLocalModelDownloadModal(currentProvider.id),
          },
        ],
      }}
      trigger={['click']}
    >
      <Button type="text" icon={<PlusOutlined />} />
    </Dropdown>
  )

  return (
    <Flex className={'flex-col gap-3'}>
      <ProviderHeader
        currentProvider={currentProvider}
        isMobile={isMobile}
        canEditProviders={canEditProviders}
        nameForm={nameForm}
        onNameChange={handleNameChange}
        onProviderToggle={handleProviderToggle}
        canEnableProvider={canEnableProvider}
        getEnableDisabledReason={getEnableDisabledReason}
      />

      {/* Downloads Section - For Local providers only */}
      {providerDownloads.length > 0 && (
        <Card title={t('providers.downloadingModels')}>
          <List
            dataSource={providerDownloads}
            renderItem={(download: DownloadInstance) => {
              const percent = download.progress_data
                ? Math.round(
                    (download.progress_data.current /
                      download.progress_data.total) *
                      100,
                  )
                : 0

              const speed = formatSpeed(download.progress_data?.download_speed)
              const eta = formatETA(download.progress_data?.eta_seconds)

              const handleCloseDownload = async (downloadId: string) => {
                try {
                  await deleteModelDownload(downloadId)
                  message.success('Download removed successfully')
                } catch (error: any) {
                  console.error('Failed to delete download:', error)
                  message.error(`Failed to remove download: ${error.message}`)
                }
              }

              return (
                <List.Item
                  actions={[
                    <Button
                      key="view"
                      type="text"
                      size="small"
                      onClick={() => openViewDownloadModal(download.id)}
                    >
                      View Details
                    </Button>,
                    download.status === 'completed' ? (
                      <Button
                        key="close"
                        type="text"
                        size="small"
                        icon={<CloseOutlined />}
                        onClick={() => handleCloseDownload(download.id)}
                        title="Remove from list"
                      >
                        Close
                      </Button>
                    ) : (
                      <Button
                        key="cancel"
                        type="text"
                        danger
                        size="small"
                        onClick={() => {
                          // TODO: Implement clearDownload when API is available
                          console.log('Clear download:', download.id)
                        }}
                      >
                        Cancel
                      </Button>
                    ),
                  ]}
                >
                  <List.Item.Meta
                    title={download.request_data.alias}
                    description={
                      <Flex vertical className="gap-1 w-full">
                        <Text type="secondary" className="text-xs">
                          {download.progress_data?.message ||
                            'Preparing download...'}
                        </Text>
                        <Progress
                          percent={percent}
                          status="active"
                          strokeColor="#1890ff"
                          size="small"
                        />
                        <Flex justify="space-between" align="center">
                          <Text type="secondary" className="text-xs">
                            {download.progress_data
                              ? `${formatBytes(download.progress_data.current)} / ${formatBytes(download.progress_data.total)}`
                              : '0 B / 0 B'}
                          </Text>
                          <Flex className={'gap-2'}>
                            {speed && (
                              <Text type="secondary" className="text-xs">
                                Speed: {speed}
                              </Text>
                            )}
                            {eta && (
                              <Text type="secondary" className="text-xs">
                                ETA: {eta}
                              </Text>
                            )}
                          </Flex>
                        </Flex>
                      </Flex>
                    }
                  />
                </List.Item>
              )
            }}
          />
        </Card>
      )}

      {/* Models Section */}
      <ModelsSection
        currentProvider={currentProvider}
        currentModels={currentModels}
        modelsLoading={modelsLoading}
        canEditProviders={canEditProviders}
        isMobile={isMobile}
        modelOperations={modelOperations}
        onAddModel={() => {
          // Not used since we have customAddButton
        }}
        onToggleModel={handleToggleModel}
        onEditModel={modelId => openEditLocalModelModal(modelId)}
        onDeleteModel={handleDeleteModel}
        onStartStopModel={handleStartStopModel}
        customAddButton={addModelButton}
      />
    </Flex>
  )
}
