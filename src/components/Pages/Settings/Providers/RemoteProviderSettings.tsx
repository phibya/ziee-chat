import {
  CopyOutlined,
  EyeInvisibleOutlined,
  EyeTwoTone,
} from '@ant-design/icons'
import { App, Button, Card, Flex, Form, Input, Typography } from 'antd'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useParams } from 'react-router-dom'
import {
  clearProvidersError,
  deleteExistingModel,
  disableModelFromUse,
  enableModelForUse,
  openAddRemoteModelDrawer,
  openEditRemoteModelDrawer,
  Stores,
  updateModelProvider,
} from '../../../../store'
import { ProviderProxySettingsForm } from './ProviderProxySettings'
import { ModelsSection } from './common/ModelsSection'
import { ProviderHeader } from './common/ProviderHeader'
import { ProxySettings } from '../../../../types'

const { Title, Text } = Typography

export function RemoteProviderSettings() {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const { providerId } = useParams<{ providerId?: string }>()

  const [form] = Form.useForm()
  const [nameForm] = Form.useForm()
  const [hasUnsavedChanges, setHasUnsavedChanges] = useState(false)
  const [pendingSettings, setPendingSettings] = useState<any>(null)

  // Store data
  const { error, modelsLoading, modelOperations } = Stores.AdminProviders

  // Get current provider and its models
  const currentProvider = Stores.AdminProviders.providers.find(
    p => p.id === providerId,
  )
  const models = currentProvider?.models || []
  const loading = modelsLoading[providerId!] || false

  // Helper functions for provider validation
  const canEnableProvider = (provider: any): boolean => {
    if (provider.enabled) return true // Already enabled
    const providerModels = provider.id === providerId ? models : []
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

  const getEnableDisabledReason = (provider: any): string | null => {
    if (provider.enabled) return null
    const providerModels = provider.id === providerId ? models : []
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

  const copyToClipboard = (text: string) => {
    if (typeof window !== 'undefined' && window.navigator?.clipboard) {
      window.navigator.clipboard.writeText(text)
      message.success(t('providers.copiedToClipboard'))
    } else {
      message.error(t('providers.clipboardNotAvailable'))
    }
  }

  const handleFormChange = (changedValues: any) => {
    if (!currentProvider) return

    setHasUnsavedChanges(true)
    setPendingSettings((prev: any) => ({ ...prev, ...changedValues }))
  }

  const handleProviderToggle = async (providerId: string, enabled: boolean) => {
    try {
      await updateModelProvider(providerId, {
        enabled: enabled,
      })
      message.success(
        `${currentProvider?.name || 'Provider'} ${enabled ? 'enabled' : 'disabled'}`,
      )
    } catch (error: any) {
      console.error('Failed to update provider:', error)
      // Handle error similar to original implementation
      if (error.response?.status === 400) {
        if (currentProvider) {
          if (models.length === 0) {
            message.error(
              `Cannot enable "${currentProvider.name}" - No models available`,
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
        const remainingEnabledModels = models.filter(
          m => m.id !== modelId && m.enabled !== false,
        )

        // If no models remain enabled and provider is currently enabled, disable the provider
        if (remainingEnabledModels.length === 0 && currentProvider.enabled) {
          try {
            await updateModelProvider(currentProvider.id, { enabled: false })
            const modelName =
              models.find(m => m.id === modelId)?.name || 'Model'
            message.success(
              `${modelName} disabled. ${currentProvider.name} provider disabled as no models remain active.`,
            )
          } catch (providerError) {
            console.error('Failed to disable provider:', providerError)
            const modelName =
              models.find(m => m.id === modelId)?.name || 'Model'
            message.warning(
              `${modelName} disabled, but failed to disable provider automatically`,
            )
          }
        } else {
          const modelName = models.find(m => m.id === modelId)?.name || 'Model'
          message.success(`${modelName} ${enabled ? 'enabled' : 'disabled'}`)
        }
      } else {
        const modelName = models.find(m => m.id === modelId)?.name || 'Model'
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

  const handleSaveSettings = async () => {
    if (!currentProvider || !pendingSettings) return

    try {
      await updateModelProvider(currentProvider.id, pendingSettings)

      setHasUnsavedChanges(false)
      setPendingSettings(null)
      message.success(t('providers.settingsSaved'))
    } catch (error) {
      console.error('Failed to save settings:', error)
      // Error is handled by the store
    }
  }

  const handleProxySettingsSave = async (proxySettings: any) => {
    if (!currentProvider) return

    try {
      await updateModelProvider(currentProvider.id, {
        proxy_settings: proxySettings,
      })
      message.success(t('providers.proxySettingsSaved'))
    } catch (error) {
      console.error('Failed to save proxy settings:', error)
      // Error is handled by the store
    }
  }

  const handleAddModel = () => {
    if (currentProvider) {
      openAddRemoteModelDrawer(currentProvider.id, currentProvider.type)
    }
  }

  // Show errors
  useEffect(() => {
    if (error) {
      message.error(error)
      clearProvidersError()
    }
  }, [error]) // Removed message from dependencies to prevent infinite rerenders

  // Update forms when provider changes
  useEffect(() => {
    if (currentProvider) {
      form.setFieldsValue({
        api_key: currentProvider.api_key,
        base_url: currentProvider.base_url,
      })
      nameForm.setFieldsValue({
        name: currentProvider.name,
      })
      // Clear unsaved changes when switching providers
      setHasUnsavedChanges(false)
      setPendingSettings(null)
    }
  }, [currentProvider]) // Removed form and nameForm from dependencies to prevent infinite rerenders

  // Return early if no provider or not remote
  if (!currentProvider || currentProvider.type === 'local') {
    return null
  }

  return (
    <Flex className={'flex-col gap-3'}>
      <ProviderHeader
        currentProvider={currentProvider}
        onProviderToggle={handleProviderToggle}
        canEnableProvider={canEnableProvider}
        getEnableDisabledReason={getEnableDisabledReason}
      />

      {/* API Configuration */}
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
                  iconRender={visible =>
                    visible ? <EyeTwoTone /> : <EyeInvisibleOutlined />
                  }
                  suffix={
                    <Button
                      type="text"
                      icon={<CopyOutlined />}
                      onClick={() =>
                        copyToClipboard(currentProvider.api_key || '')
                      }
                    />
                  }
                />
              </Form.Item>
            </div>

            <div>
              <Title level={5}>Base URL</Title>
              <Text type="secondary">
                The base{' '}
                {currentProvider.type === 'gemini' ? 'OpenAI-compatible' : ''}{' '}
                endpoint to use. See the{' '}
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

      {/* Models Section */}
      <ModelsSection
        currentProvider={currentProvider}
        currentModels={models}
        modelsLoading={loading}
        modelOperations={modelOperations}
        onAddModel={handleAddModel}
        onToggleModel={handleToggleModel}
        onEditModel={modelId => openEditRemoteModelDrawer(modelId)}
        onDeleteModel={handleDeleteModel}
      />

      {/* Proxy Settings - For non-Local providers */}
      <ProviderProxySettingsForm
        initialSettings={
          currentProvider.proxy_settings || ({} as ProxySettings)
        }
        onSave={handleProxySettingsSave}
      />
    </Flex>
  )
}
