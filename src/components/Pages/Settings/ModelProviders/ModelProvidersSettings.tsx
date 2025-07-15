import {
  App,
  Button,
  Card,
  Divider,
  Dropdown,
  Empty,
  Flex,
  Form,
  Input,
  Layout,
  List,
  Menu,
  Modal,
  Space,
  Spin,
  Switch,
  Tooltip,
  Typography,
} from 'antd'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  CopyOutlined,
  DeleteOutlined,
  DownOutlined,
  EditOutlined,
  EyeInvisibleOutlined,
  EyeTwoTone,
  MenuOutlined,
  PlusOutlined,
  SettingOutlined,
} from '@ant-design/icons'
import { useShallow } from 'zustand/react/shallow'
import { Permission, usePermissions } from '../../../../permissions'
import { isDesktopApp } from '../../../../api/core'
import {
  ModelProvider,
  ModelProviderModel,
  ModelProviderType,
} from '../../../../types/api/modelProvider'
import { AddProviderModal } from './AddProviderModal'
import { AddModelModal } from './AddModelModal'
import { EditModelModal } from './EditModelModal'
import { ModelProviderProxySettingsForm } from './ModelProviderProxySettings'
import { CandleConfigurationSection } from './shared/CandleConfigurationSection'
import { useModelProvidersStore } from '../../../../store/modelProviders'

const { Title, Text } = Typography
const { Sider, Content } = Layout

const PROVIDER_ICONS: Record<ModelProviderType, string> = {
  candle: '🕯',
  openai: '🤖',
  anthropic: '🤖',
  groq: '⚡',
  gemini: '💎',
  mistral: '🌊',
  custom: '🔧',
}

export function ModelProvidersSettings() {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const { hasPermission } = usePermissions()

  // Model providers store
  const {
    providers,
    loading,
    error,
    loadProviders,
    createProvider,
    updateProvider,
    deleteProvider,
    cloneProvider,
    addModel,
    updateModel,
    deleteModel,
    startModel,
    stopModel,
    enableModel,
    disableModel,
    clearError,
  } = useModelProvidersStore(
    useShallow(state => ({
      providers: state.providers,
      loading: state.loading,
      creating: state.creating,
      updating: state.updating,
      deleting: state.deleting,
      error: state.error,
      loadProviders: state.loadProviders,
      createProvider: state.createProvider,
      updateProvider: state.updateProvider,
      deleteProvider: state.deleteProvider,
      cloneProvider: state.cloneProvider,
      addModel: state.addModel,
      updateModel: state.updateModel,
      deleteModel: state.deleteModel,
      startModel: state.startModel,
      stopModel: state.stopModel,
      enableModel: state.enableModel,
      disableModel: state.disableModel,
      clearError: state.clearError,
    })),
  )

  const [selectedProvider, setSelectedProvider] = useState<string>('')
  const [form] = Form.useForm()
  const [nameForm] = Form.useForm()
  const [isMobile, setIsMobile] = useState(false)
  const [isAddModalOpen, setIsAddModalOpen] = useState(false)
  const [isAddModelModalOpen, setIsAddModelModalOpen] = useState(false)
  const [isEditModelModalOpen, setIsEditModelModalOpen] = useState(false)
  const [selectedModel, setSelectedModel] = useState<ModelProviderModel | null>(
    null,
  )
  const [hasUnsavedChanges, setHasUnsavedChanges] = useState(false)
  const [pendingSettings, setPendingSettings] = useState<any>(null)

  // Check permissions for web app
  const canEditProviders =
    isDesktopApp || hasPermission(Permission.config.modelProviders.edit)
  const canViewProviders =
    isDesktopApp || hasPermission(Permission.config.modelProviders.read)

  // If user doesn't have view permissions, don't render the component
  if (!canViewProviders) {
    return (
      <div style={{ padding: '24px', textAlign: 'center' }}>
        <Title level={3}>Access Denied</Title>
        <Text type="secondary">
          You do not have permission to view model provider settings.
        </Text>
      </div>
    )
  }

  useEffect(() => {
    const checkMobile = () => {
      setIsMobile(window.innerWidth < 768)
    }

    checkMobile()
    window.addEventListener('resize', checkMobile)

    return () => window.removeEventListener('resize', checkMobile)
  }, [])

  const currentProvider = providers.find(p => p.id === selectedProvider)

  const canEnableProvider = (provider: ModelProvider): boolean => {
    if (provider.enabled) return true // Already enabled
    if (provider.models.length === 0) return false
    if (provider.type === 'candle') return true
    if (!provider.api_key || provider.api_key.trim() === '') return false
    if (!provider.base_url || provider.base_url.trim() === '') return false
    try {
      new globalThis.URL(provider.base_url)
      return true
    } catch {
      return false
    }
  }

  const getEnableDisabledReason = (provider: ModelProvider): string | null => {
    if (provider.enabled) return null
    if (provider.models.length === 0)
      return 'No models available. Add at least one model first.'
    if (provider.type === 'candle') return null
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

  useEffect(() => {
    loadProviders()
  }, [loadProviders])

  // Show errors
  useEffect(() => {
    if (error) {
      message.error(error)
      clearError()
    }
  }, [error, message, clearError])

  // Set first provider as selected when providers load
  useEffect(() => {
    if (providers.length > 0 && !selectedProvider) {
      setSelectedProvider(providers[0].id)
    }
  }, [providers, selectedProvider])

  useEffect(() => {
    if (currentProvider) {
      form.setFieldsValue({
        api_key: currentProvider.api_key,
        base_url: currentProvider.base_url,
        settings: currentProvider.settings,
      })
      nameForm.setFieldsValue({
        name: currentProvider.name,
      })
      // Clear unsaved changes when switching providers
      setHasUnsavedChanges(false)
      setPendingSettings(null)
    }
  }, [currentProvider, form, nameForm])

  const handleProviderToggle = async (providerId: string, enabled: boolean) => {
    if (!canEditProviders) {
      message.error(t('modelProviders.noPermissionModify'))
      return
    }

    try {
      const updatedProvider = await updateProvider(providerId, {
        enabled: enabled,
      })
      message.success(
        `${updatedProvider.name} ${enabled ? 'enabled' : 'disabled'}`,
      )
    } catch (error: any) {
      console.error('Failed to update provider:', error)
      if (error.response?.status === 400) {
        const provider = providers.find(p => p.id === providerId)
        if (provider) {
          if (provider.models.length === 0) {
            message.error(
              `Cannot enable "${provider.name}" - No models available`,
            )
          } else if (
            provider.type !== 'candle' &&
            (!provider.api_key || provider.api_key.trim() === '')
          ) {
            message.error(
              `Cannot enable "${provider.name}" - API key is required`,
            )
          } else if (
            provider.type !== 'candle' &&
            (!provider.base_url || provider.base_url.trim() === '')
          ) {
            message.error(
              `Cannot enable "${provider.name}" - Base URL is required`,
            )
          } else {
            message.error(
              `Cannot enable "${provider.name}" - Invalid base URL format`,
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

  const handleFormChange = (changedValues: any) => {
    if (!currentProvider || !canEditProviders) return

    setHasUnsavedChanges(true)
    setPendingSettings((prev: any) => ({ ...prev, ...changedValues }))
  }

  const handleNameChange = async (changedValues: any) => {
    if (!currentProvider || !canEditProviders) return

    try {
      await updateProvider(currentProvider.id, { name: changedValues.name })
    } catch (error) {
      console.error('Failed to update provider:', error)
      // Error is handled by the store
    }
  }

  const handleSettingsChange = (changedValues: any) => {
    if (!currentProvider || !canEditProviders) return

    setHasUnsavedChanges(true)
    setPendingSettings((prev: any) => ({
      ...prev,
      settings: { ...currentProvider.settings, ...changedValues },
    }))
  }

  const handleSaveSettings = async () => {
    if (!currentProvider || !canEditProviders || !pendingSettings) return

    try {
      await updateProvider(currentProvider.id, pendingSettings)

      // Update form with the new values from updated provider
      const updatedProvider = providers.find(p => p.id === currentProvider.id)
      if (updatedProvider) {
        form.setFieldsValue({
          api_key: updatedProvider.api_key,
          base_url: updatedProvider.base_url,
          settings: updatedProvider.settings,
        })
      }

      setHasUnsavedChanges(false)
      setPendingSettings(null)
      message.success(t('modelProviders.settingsSaved'))
    } catch (error) {
      console.error('Failed to save settings:', error)
      // Error is handled by the store
    }
  }

  const handleProxySettingsSave = async (proxySettings: any) => {
    if (!currentProvider || !canEditProviders) return

    try {
      await updateProvider(currentProvider.id, {
        proxy_settings: proxySettings,
      })
      message.success(t('modelProviders.proxySettingsSaved'))
    } catch (error) {
      console.error('Failed to save proxy settings:', error)
      // Error is handled by the store
    }
  }

  const handleDeleteProvider = async (providerId: string) => {
    if (!canEditProviders) {
      message.error(t('modelProviders.noPermissionDelete'))
      return
    }

    const provider = providers.find(p => p.id === providerId)
    if (!provider) return

    Modal.confirm({
      title: t('modelProviders.deleteModelProvider'),
      content: `Are you sure you want to delete "${provider.name}"? This action cannot be undone.`,
      okText: 'Delete',
      okType: 'danger',
      cancelText: 'Cancel',
      onOk: async () => {
        try {
          await deleteProvider(providerId)
          if (selectedProvider === providerId) {
            const remainingProviders = providers.filter(
              p => p.id !== providerId,
            )
            setSelectedProvider(
              remainingProviders.length > 0 ? remainingProviders[0].id : '',
            )
          }
          message.success(t('modelProviders.providerDeleted'))
        } catch (error: any) {
          console.error('Failed to delete provider:', error)
          // Error is handled by the store
        }
      },
    })
  }

  const handleCloneProvider = async (providerId: string) => {
    if (!canEditProviders) {
      message.error(t('modelProviders.noPermissionClone'))
      return
    }

    try {
      await cloneProvider(
        providerId,
        `${providers.find(p => p.id === providerId)?.name} (Clone)`,
      )
      message.success(t('modelProviders.providerCloned'))
    } catch (error) {
      console.error('Failed to clone provider:', error)
      // Error is handled by the store
    }
  }

  const handleAddProvider = async (providerData: any) => {
    try {
      await createProvider(providerData)
      setIsAddModalOpen(false)
      message.success(t('modelProviders.providerAdded'))
    } catch (error: any) {
      console.error('Failed to add provider:', error)
      // Error is handled by the store
    }
  }

  const handleAddModel = async (modelData: any) => {
    if (!currentProvider) return

    try {
      // Handle Candle uploads differently - they're already processed
      if (modelData.type === 'candle-upload') {
        // For Candle uploads, just refresh the providers list
        await loadProviders()
        setIsAddModelModalOpen(false)
        message.success(t('modelProviders.modelAdded'))
      } else {
        // For regular models, add them normally
        await addModel(currentProvider.id, modelData)

        // Refresh providers list to include any uploaded models
        await loadProviders()

        setIsAddModelModalOpen(false)
        message.success(t('modelProviders.modelAdded'))
      }
    } catch (error) {
      console.error('Failed to add model:', error)
      // Error is handled by the store
    }
  }

  const handleEditModel = async (modelData: any) => {
    if (!currentProvider || !selectedModel) return

    try {
      await updateModel(modelData.id, modelData)
      setIsEditModelModalOpen(false)
      message.success(t('modelProviders.modelUpdated'))
    } catch (error) {
      console.error('Failed to update model:', error)
      // Error is handled by the store
    }
  }

  const handleDeleteModel = async (modelId: string) => {
    if (!currentProvider) return

    try {
      await deleteModel(modelId)
      message.success(t('modelProviders.modelDeleted'))
    } catch (error) {
      console.error('Failed to delete model:', error)
      // Error is handled by the store
    }
  }

  const handleToggleModel = async (modelId: string, enabled: boolean) => {
    if (!currentProvider) return

    try {
      if (enabled) {
        await enableModel(modelId)
      } else {
        await disableModel(modelId)
      }

      // Check if this was the last enabled model being disabled
      if (!enabled) {
        const currentModels = currentProvider.models
        const remainingEnabledModels = currentModels.filter(
          m => m.id !== modelId && m.enabled !== false,
        )

        // If no models remain enabled and provider is currently enabled, disable the provider
        if (remainingEnabledModels.length === 0 && currentProvider.enabled) {
          try {
            await updateProvider(currentProvider.id, { enabled: false })
            const modelName =
              currentModels.find(m => m.id === modelId)?.name || 'Model'
            message.success(
              `${modelName} disabled. ${currentProvider.name} provider disabled as no models remain active.`,
            )
          } catch (providerError) {
            console.error('Failed to disable provider:', providerError)
            const modelName =
              currentModels.find(m => m.id === modelId)?.name || 'Model'
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
          currentProvider.models.find(m => m.id === modelId)?.name || 'Model'
        message.success(`${modelName} ${enabled ? 'enabled' : 'disabled'}`)
      }
    } catch (error) {
      console.error('Failed to toggle model:', error)
      // Error is handled by the store
    }
  }

  const handleStartStopModel = async (modelId: string, isActive: boolean) => {
    if (!currentProvider || currentProvider.type !== 'candle') return

    try {
      if (isActive) {
        await startModel(modelId)
      } else {
        await stopModel(modelId)
      }

      const modelName =
        currentProvider.models.find(m => m.id === modelId)?.name || 'Model'
      message.success(`${modelName} ${isActive ? 'started' : 'stopped'}`)
    } catch (error) {
      console.error('Failed to start/stop model:', error)
      // Error is handled by the store
    }
  }

  const copyToClipboard = (text: string) => {
    if (typeof window !== 'undefined' && window.navigator?.clipboard) {
      window.navigator.clipboard.writeText(text)
      message.success(t('modelProviders.copiedToClipboard'))
    } else {
      message.error(t('modelProviders.clipboardNotAvailable'))
    }
  }

  const getProviderActions = (provider: ModelProvider) => {
    const actions: any[] = []

    if (canEditProviders) {
      // actions.push({
      //   key: 'edit',
      //   icon: <EditOutlined />,
      //   label: 'Edit',
      //   onClick: () => {
      //     setSelectedProvider(provider.id)
      //   },
      // })

      actions.push({
        key: 'clone',
        icon: <CopyOutlined />,
        label: t('buttons.clone'),
        onClick: () => handleCloneProvider(provider.id),
      })

      actions.push({
        key: 'delete',
        icon: <DeleteOutlined />,
        label: t('buttons.delete'),
        onClick: () => handleDeleteProvider(provider.id),
        disabled: provider.is_default,
      })
    }

    return actions
  }

  const menuItems = providers.map(provider => ({
    key: provider.id,
    label: (
      <Flex className={'flex-row gap-2 items-center'}>
        <span className={'text-lg'}>{PROVIDER_ICONS[provider.type]}</span>
        <div className={'flex-1'}>
          <Typography.Text>{provider.name}</Typography.Text>
        </div>
        {canEditProviders && (
          <Dropdown
            menu={{ items: getProviderActions(provider) }}
            trigger={['click']}
          >
            <Button
              type="text"
              icon={<MenuOutlined />}
              size="small"
              onClick={(e: React.MouseEvent) => e.stopPropagation()}
            />
          </Dropdown>
        )}
      </Flex>
    ),
  }))

  if (canEditProviders) {
    menuItems.push({
      key: 'add-provider',
      //@ts-ignore
      icon: <PlusOutlined />,
      label: <Typography.Text>Add Provider</Typography.Text>,
    })
  }

  const ProviderMenu = () => (
    <Menu
      selectedKeys={[selectedProvider]}
      items={menuItems}
      onClick={({ key }) => {
        if (key === 'add-provider') {
          setIsAddModalOpen(true)
        } else {
          setSelectedProvider(key)
        }
      }}
      className={'!bg-transparent'}
    />
  )

  const renderProviderSettings = () => {
    if (loading) {
      return (
        <div style={{ textAlign: 'center', padding: '50px' }}>
          <Spin size="large" />
        </div>
      )
    }

    if (!currentProvider) {
      return (
        <Empty
          description={t('modelProviders.noProviderSelected')}
          image={Empty.PRESENTED_IMAGE_SIMPLE}
        />
      )
    }

    return (
      <Flex className={'flex-col gap-3'}>
        {/* Provider Header - Hide on mobile since it's shown in dropdown */}
        {!isMobile && (
          <Flex justify="space-between" align="center">
            <Flex align="center" gap="middle">
              <span style={{ fontSize: '24px' }}>
                {PROVIDER_ICONS[currentProvider.type]}
              </span>
              <Form
                form={nameForm}
                layout="inline"
                initialValues={{ name: currentProvider.name }}
                onValuesChange={handleNameChange}
              >
                <Form.Item name="name" style={{ margin: 0 }}>
                  <Input
                    variant="borderless"
                    style={{
                      fontSize: '24px',
                      fontWeight: 600,
                      padding: 0,
                      border: 'none',
                      boxShadow: 'none',
                    }}
                    disabled={!canEditProviders}
                  />
                </Form.Item>
              </Form>
            </Flex>
            {(() => {
              const disabledReason = getEnableDisabledReason(currentProvider)
              const switchElement = (
                <Switch
                  checked={currentProvider.enabled}
                  disabled={
                    !canEditProviders ||
                    (!currentProvider.enabled &&
                      !canEnableProvider(currentProvider))
                  }
                  onChange={enabled =>
                    handleProviderToggle(currentProvider.id, enabled)
                  }
                />
              )

              if (!canEditProviders) return switchElement
              if (disabledReason && !currentProvider.enabled) {
                return <Tooltip title={disabledReason}>{switchElement}</Tooltip>
              }
              return switchElement
            })()}
          </Flex>
        )}

        {/* Mobile Provider Header */}
        {isMobile && (
          <Flex className={'flex-col gap-2'}>
            <Form
              form={nameForm}
              layout="vertical"
              initialValues={{ name: currentProvider.name }}
              onValuesChange={handleNameChange}
            >
              <Form.Item
                name="name"
                label={t('modelProviders.providerName')}
                style={{ margin: 0 }}
              >
                <Input
                  style={{
                    fontSize: '16px',
                    fontWeight: 600,
                  }}
                  disabled={!canEditProviders}
                />
              </Form.Item>
            </Form>
            <Flex justify="space-between" align="center">
              <Text strong style={{ fontSize: '16px' }}>
                Enable Provider
              </Text>
              {(() => {
                const disabledReason = getEnableDisabledReason(currentProvider)
                const switchElement = (
                  <Switch
                    checked={currentProvider.enabled}
                    disabled={
                      !canEditProviders ||
                      (!currentProvider.enabled &&
                        !canEnableProvider(currentProvider))
                    }
                    onChange={enabled =>
                      handleProviderToggle(currentProvider.id, enabled)
                    }
                  />
                )

                if (!canEditProviders) return switchElement
                if (disabledReason && !currentProvider.enabled) {
                  return (
                    <Tooltip title={disabledReason}>{switchElement}</Tooltip>
                  )
                }
                return switchElement
              })()}
            </Flex>
          </Flex>
        )}

        {/* API Configuration */}
        {currentProvider.type !== 'candle' && (
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
              title={t('modelProviders.apiConfiguration')}
              extra={
                canEditProviders && (
                  <Button
                    type="primary"
                    onClick={handleSaveSettings}
                    disabled={!hasUnsavedChanges}
                  >
                    Save
                  </Button>
                )
              }
            >
              <Flex className={'flex-col gap-2'}>
                <div>
                  <Title level={5}>API Key</Title>
                  <Text type="secondary">
                    The {currentProvider.name} API uses API keys for
                    authentication. Visit your{' '}
                    <Text type="danger">API Keys</Text> page to retrieve the API
                    key you'll use in your requests.
                  </Text>
                  <Form.Item
                    name="api_key"
                    style={{ marginBottom: 0, marginTop: 16 }}
                  >
                    <Input.Password
                      placeholder={t('modelProviders.insertApiKey')}
                      disabled={!canEditProviders}
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

                <Divider style={{ margin: 0 }} />

                <div>
                  <Title level={5}>Base URL</Title>
                  <Text type="secondary">
                    The base{' '}
                    {currentProvider.type === 'gemini'
                      ? 'OpenAI-compatible'
                      : ''}{' '}
                    endpoint to use. See the{' '}
                    <Text type="danger">
                      {currentProvider.name} documentation
                    </Text>{' '}
                    for more information.
                  </Text>
                  <Form.Item
                    name="base_url"
                    style={{ marginBottom: 0, marginTop: 16 }}
                  >
                    <Input
                      placeholder={t('modelProviders.baseUrl')}
                      disabled={!canEditProviders}
                    />
                  </Form.Item>
                </div>
              </Flex>
            </Card>
          </Form>
        )}

        {/* Models Section */}
        <Card
          title={t('modelProviders.models')}
          extra={
            canEditProviders && (
              <Button
                type="text"
                icon={<PlusOutlined />}
                onClick={() => setIsAddModelModalOpen(true)}
              />
            )
          }
        >
          {currentProvider.type === 'candle' && (
            <Flex
              justify="space-between"
              align={isMobile ? 'flex-start' : 'center'}
              vertical={isMobile}
              gap={isMobile ? 'small' : 0}
              style={{ marginBottom: 16 }}
            >
              <Text>Import models from your local machine</Text>
              <Button
                icon={<PlusOutlined />}
                block={isMobile}
                disabled={!canEditProviders}
                onClick={() => setIsAddModelModalOpen(true)}
              >
                Import
              </Button>
            </Flex>
          )}

          <List
            dataSource={currentProvider.models}
            locale={{ emptyText: 'No models added yet' }}
            renderItem={model => (
              <List.Item
                actions={
                  canEditProviders
                    ? [
                        currentProvider.type === 'candle' &&
                          currentProvider.enabled && (
                            <Button
                              key="start-stop"
                              type={model.isActive ? 'default' : 'primary'}
                              size={isMobile ? 'small' : 'middle'}
                              onClick={() =>
                                handleStartStopModel(model.id, !model.isActive)
                              }
                            >
                              {model.isActive ? 'Stop' : 'Start'}
                            </Button>
                          ),
                        <Button
                          key="edit"
                          type="text"
                          icon={<EditOutlined />}
                          size={isMobile ? 'small' : 'middle'}
                          onClick={() => {
                            setSelectedModel(model)
                            setIsEditModelModalOpen(true)
                          }}
                        >
                          {!isMobile && 'Edit'}
                        </Button>,
                        <Button
                          key="delete"
                          type="text"
                          icon={<DeleteOutlined />}
                          size={isMobile ? 'small' : 'middle'}
                          onClick={() => handleDeleteModel(model.id)}
                        >
                          {!isMobile && 'Delete'}
                        </Button>,
                      ].filter(Boolean)
                    : []
                }
              >
                <List.Item.Meta
                  avatar={
                    <Switch
                      checked={model.enabled !== false}
                      onChange={checked => handleToggleModel(model.id, checked)}
                      disabled={!canEditProviders}
                    />
                  }
                  title={
                    <Flex align="center" gap="small">
                      <Text>{model.alias}</Text>
                      {model.isDeprecated && (
                        <span style={{ fontSize: '12px' }}>⚠️</span>
                      )}
                    </Flex>
                  }
                  description={
                    <Space direction="vertical" size="small">
                      <Text type="secondary" className="text-xs">
                        Model ID: {model.name}
                      </Text>
                      {model.description && (
                        <Text type="secondary">{model.description}</Text>
                      )}
                      {model.capabilities && (
                        <Space size="small" wrap>
                          {model.capabilities.vision && (
                            <Text type="secondary">👁️ Vision</Text>
                          )}
                          {model.capabilities.audio && (
                            <Text type="secondary">🎵 Audio</Text>
                          )}
                          {model.capabilities.tools && (
                            <Text type="secondary">🔧 Tools</Text>
                          )}
                          {model.capabilities.codeInterpreter && (
                            <Text type="secondary">💻 Code</Text>
                          )}
                        </Space>
                      )}
                    </Space>
                  }
                />
              </List.Item>
            )}
          />
        </Card>

        {/* Proxy Settings - For non-Candle providers */}
        {currentProvider.type !== 'candle' &&
          currentProvider.proxy_settings && (
            <ModelProviderProxySettingsForm
              providerId={currentProvider.id}
              initialSettings={currentProvider.proxy_settings}
              onSave={handleProxySettingsSave}
              disabled={!canEditProviders}
            />
          )}

        {/* Candle Specific Settings */}
        {currentProvider.type === 'candle' && currentProvider.settings && (
          <Form
            layout="vertical"
            initialValues={currentProvider.settings}
            onValuesChange={handleSettingsChange}
          >
            <Card
              title={t('modelProviders.configuration')}
              extra={
                canEditProviders && (
                  <Button
                    type="primary"
                    onClick={handleSaveSettings}
                    disabled={!hasUnsavedChanges}
                  >
                    Save
                  </Button>
                )
              }
            >
              <CandleConfigurationSection
                disabled={!canEditProviders}
                useNestedSettings={false}
                wrapInCard={false}
              />
            </Card>
          </Form>
        )}
      </Flex>
    )
  }

  return (
    <Layout>
      {/* Desktop Sidebar */}
      {!isMobile && (
        <Sider
          width={200}
          theme="light"
          style={{ backgroundColor: 'transparent' }}
        >
          <div>
            <Title level={3}>Model Providers</Title>
            <ProviderMenu />
          </div>
        </Sider>
      )}

      {/* Main Content */}
      <Layout className={'px-2'}>
        <Content>
          {/* Mobile Header with Provider Selector */}
          {isMobile && (
            <div style={{ marginBottom: '24px' }}>
              <Title level={3} style={{ margin: '0 0 16px 0' }}>
                <SettingOutlined style={{ marginRight: 8 }} />
                Model Providers
              </Title>
              <Dropdown
                menu={{
                  items: menuItems,
                  onClick: ({ key }) => {
                    if (key === 'add-provider') {
                      setIsAddModalOpen(true)
                    } else {
                      setSelectedProvider(key)
                    }
                  },
                }}
                trigger={['click']}
              >
                <Button
                  size="large"
                  style={{ width: '100%', textAlign: 'left' }}
                >
                  <Flex justify="space-between" align="center">
                    <Flex align="center" gap="middle">
                      <span style={{ fontSize: '20px' }}>
                        {currentProvider
                          ? PROVIDER_ICONS[currentProvider.type]
                          : ''}
                      </span>
                      <span>{currentProvider?.name}</span>
                    </Flex>
                    <DownOutlined />
                  </Flex>
                </Button>
              </Dropdown>
            </div>
          )}
          {renderProviderSettings()}
        </Content>
      </Layout>

      {/* Modals */}
      <AddProviderModal
        open={isAddModalOpen}
        onClose={() => setIsAddModalOpen(false)}
        onSubmit={handleAddProvider}
      />

      <AddModelModal
        open={isAddModelModalOpen}
        providerId={selectedProvider}
        providerType={currentProvider?.type || 'custom'}
        onClose={() => setIsAddModelModalOpen(false)}
        onSubmit={handleAddModel}
      />

      <EditModelModal
        open={isEditModelModalOpen}
        model={selectedModel}
        providerType={currentProvider?.type || 'custom'}
        onClose={() => {
          setIsEditModelModalOpen(false)
          setSelectedModel(null)
        }}
        onSubmit={handleEditModel}
      />
    </Layout>
  )
}
