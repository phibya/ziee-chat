import { App, Button, Card, Flex, Form, Input, Typography } from 'antd'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useParams } from 'react-router-dom'
import {
  clearRAGProvidersError,
  deleteSystemRAGInstance,
  disableSystemRAGInstance,
  enableSystemRAGInstance,
  getInstancesForProvider,
  openAddSystemInstanceDrawer,
  openEditSystemInstanceDrawer,
  Stores,
  updateRAGProvider,
} from '../../../../store'
import { RAGProvider } from '../../../../types/api'
import { RAGProviderHeader } from './RAGProviderHeader'
import { SystemInstancesSection } from './SystemInstancesSection'

const { Title, Text } = Typography

export function RAGProviderSettings() {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const { providerId } = useParams<{ providerId?: string }>()

  const [form] = Form.useForm()
  const [hasUnsavedChanges, setHasUnsavedChanges] = useState(false)
  const [pendingSettings, setPendingSettings] = useState<any>(null)

  // Store data
  const { error, instanceOperations, instancesLoading } = Stores.AdminRAGProviders

  // Get current provider and its instances
  const currentProvider = Stores.AdminRAGProviders.providers.find(
    p => p.id === providerId,
  )
  const instances = getInstancesForProvider(providerId || '')
  const loading = instancesLoading[providerId!] || false

  // Helper functions for provider validation
  const canEnableProvider = (provider: RAGProvider): boolean => {
    if (provider.enabled) return true // Already enabled
    const providerInstances = provider.id === providerId ? instances : []
    if (providerInstances.length === 0) return false
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

  const getEnableDisabledReason = (provider: RAGProvider): string | null => {
    if (provider.enabled) return null
    const providerInstances = provider.id === providerId ? instances : []
    if (providerInstances.length === 0)
      return 'No instances available. Add at least one instance first.'
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

  const handleFormChange = (changedValues: any) => {
    if (!currentProvider) return

    setHasUnsavedChanges(true)
    setPendingSettings((prev: any) => ({ ...prev, ...changedValues }))
  }

  const handleProviderToggle = async (providerId: string, enabled: boolean) => {
    try {
      await updateRAGProvider(providerId, {
        enabled: enabled,
      })
      message.success(
        `${currentProvider?.name || 'Provider'} ${enabled ? 'enabled' : 'disabled'}`,
      )
    } catch (error: any) {
      console.error('Failed to update RAG provider:', error)
      // Handle error similar to original implementation
      if (error.response?.status === 400) {
        if (currentProvider) {
          if (instances.length === 0) {
            message.error(
              `Cannot enable "${currentProvider.name}" - No instances available`,
            )
          } else if (
            currentProvider.type !== 'local' &&
            (!currentProvider.api_key || currentProvider.api_key.trim() === '')
          ) {
            message.error(
              `Cannot enable "${currentProvider.name}" - API key is required`,
            )
          } else if (
            currentProvider.type !== 'local' &&
            (!currentProvider.base_url ||
              currentProvider.base_url.trim() === '')
          ) {
            message.error(
              `Cannot enable "${currentProvider.name}" - Base URL is required`,
            )
          } else {
            message.error(
              `Cannot enable "${currentProvider.name}" - Invalid base URL format`,
            )
          }
        } else {
          message.error(error?.message || 'Failed to update RAG provider')
        }
      } else {
        message.error(error?.message || 'Failed to update RAG provider')
      }
    }
  }

  const handleToggleInstance = async (instanceId: string, enabled: boolean) => {
    if (!currentProvider) return

    try {
      if (enabled) {
        await enableSystemRAGInstance(instanceId)
      } else {
        await disableSystemRAGInstance(instanceId)
      }

      // Check if this was the last enabled instance being disabled
      if (!enabled) {
        const remainingEnabledInstances = instances.filter(
          i => i.id !== instanceId && i.enabled !== false,
        )

        // If no instances remain enabled and provider is currently enabled, disable the provider
        if (remainingEnabledInstances.length === 0 && currentProvider.enabled) {
          try {
            await updateRAGProvider(currentProvider.id, { enabled: false })
            const instanceName =
              instances.find(i => i.id === instanceId)?.name || 'Instance'
            message.success(
              `${instanceName} disabled. ${currentProvider.name} provider disabled as no instances remain active.`,
            )
          } catch (providerError) {
            console.error('Failed to disable RAG provider:', providerError)
            const instanceName =
              instances.find(i => i.id === instanceId)?.name || 'Instance'
            message.warning(
              `${instanceName} disabled, but failed to disable provider automatically`,
            )
          }
        } else {
          const instanceName = instances.find(i => i.id === instanceId)?.name || 'Instance'
          message.success(`${instanceName} ${enabled ? 'enabled' : 'disabled'}`)
        }
      } else {
        const instanceName = instances.find(i => i.id === instanceId)?.name || 'Instance'
        message.success(`${instanceName} ${enabled ? 'enabled' : 'disabled'}`)
      }
    } catch (error) {
      console.error('Failed to toggle instance:', error)
      // Error is handled by the store
    }
  }

  const handleDeleteInstance = async (instanceId: string) => {
    if (!currentProvider) return

    try {
      await deleteSystemRAGInstance(instanceId)
      message.success(t('providers.instanceDeleted'))
    } catch (error) {
      console.error('Failed to delete instance:', error)
      // Error is handled by the store
    }
  }

  const handleSaveSettings = async () => {
    if (!currentProvider || !pendingSettings) return

    try {
      await updateRAGProvider(currentProvider.id, pendingSettings)

      setHasUnsavedChanges(false)
      setPendingSettings(null)
      message.success(t('providers.settingsSaved'))
    } catch (error) {
      console.error('Failed to save settings:', error)
      // Error is handled by the store
    }
  }

  const handleAddInstance = () => {
    if (currentProvider) {
      openAddSystemInstanceDrawer(currentProvider.id)
    }
  }

  // Show errors
  useEffect(() => {
    if (error) {
      message.error(error)
      clearRAGProvidersError()
    }
  }, [error])

  // Update forms when provider changes
  useEffect(() => {
    if (currentProvider) {
      form.setFieldsValue({
        api_key: currentProvider.api_key,
        base_url: currentProvider.base_url,
      })
      // Clear unsaved changes when switching providers
      setHasUnsavedChanges(false)
      setPendingSettings(null)
    }
  }, [currentProvider])

  // Return early if no provider
  if (!currentProvider) {
    return null
  }

  return (
    <Flex className={'flex-col gap-3'}>
      <RAGProviderHeader
        currentProvider={currentProvider}
        onProviderToggle={handleProviderToggle}
        canEnableProvider={canEnableProvider}
        getEnableDisabledReason={getEnableDisabledReason}
      />

      {/* API Configuration - Only for non-local providers */}
      {currentProvider.type !== 'local' && (
        <Form
          form={form}
          layout="vertical"
          initialValues={{
            api_key: currentProvider.api_key,
            base_url: currentProvider.base_url,
          }}
          onValuesChange={handleFormChange}
        >
          <Card
            title={t('providers.apiConfiguration')}
            extra={
              <Button
                type="primary"
                onClick={handleSaveSettings}
                disabled={!hasUnsavedChanges}
              >
                Save
              </Button>
            }
          >
            <Flex className={'flex-col gap-3'}>
              <div>
                <Title level={5}>API Key</Title>
                <Text type="secondary">
                  The {currentProvider.name} API uses API keys for authentication.
                  Visit your <Text type="danger">API Keys</Text> page to retrieve
                  the API key you'll use in your requests.
                </Text>
                <Form.Item
                  name="api_key"
                  style={{ marginBottom: 0, marginTop: 16 }}
                >
                  <Input.Password
                    placeholder={t('providers.insertApiKey')}
                  />
                </Form.Item>
              </div>

              <div>
                <Title level={5}>Base URL</Title>
                <Text type="secondary">
                  The base endpoint to use. See the{' '}
                  <Text type="danger">{currentProvider.name} documentation</Text>{' '}
                  for more information.
                </Text>
                <Form.Item
                  name="base_url"
                  style={{ marginBottom: 0, marginTop: 16 }}
                >
                  <Input placeholder={t('providers.baseUrl')} />
                </Form.Item>
              </div>
            </Flex>
          </Card>
        </Form>
      )}

      {/* Instances Section */}
      <SystemInstancesSection
        currentProvider={currentProvider}
        currentInstances={instances}
        instancesLoading={loading}
        instanceOperations={instanceOperations}
        onAddInstance={handleAddInstance}
        onToggleInstance={handleToggleInstance}
        onEditInstance={instanceId => openEditSystemInstanceDrawer(instanceId)}
        onDeleteInstance={handleDeleteInstance}
      />
    </Flex>
  )
}