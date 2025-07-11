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
  InputNumber,
  Layout,
  List,
  Menu,
  Modal,
  Select,
  Space,
  Spin,
  Switch,
  Tooltip,
  Typography,
} from 'antd'
import { useEffect, useState } from 'react'
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
import { ApiClient } from '../../../../api/client'

const { Title, Text } = Typography
const { Sider, Content } = Layout

const PROVIDER_ICONS: Record<ModelProviderType, string> = {
  'llama.cpp': '🦙',
  openai: '🤖',
  anthropic: '🤖',
  groq: '⚡',
  gemini: '💎',
  mistral: '🌊',
  custom: '🔧',
}

export function ModelProvidersSettings() {
  const { message } = App.useApp()
  const { hasPermission } = usePermissions()
  const [providers, setProviders] = useState<ModelProvider[]>([])
  const [selectedProvider, setSelectedProvider] = useState<string>('')
  const [form] = Form.useForm()
  const [nameForm] = Form.useForm()
  const [isMobile, setIsMobile] = useState(false)
  const [loading, setLoading] = useState(true)
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
    if (provider.type === 'llama.cpp') return true
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
    if (provider.type === 'llama.cpp') return null
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
  }, [])

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

  const loadProviders = async () => {
    try {
      setLoading(true)
      const response = await ApiClient.ModelProviders.list({})
      setProviders(response.providers)
      if (response.providers.length > 0) {
        setSelectedProvider(response.providers[0].id)
      }
    } catch (error) {
      console.error('Failed to load providers:', error)
      message.error('Failed to load model providers')
    } finally {
      setLoading(false)
    }
  }

  const handleProviderToggle = async (providerId: string, enabled: boolean) => {
    if (!canEditProviders) {
      message.error('You do not have permission to modify provider settings')
      return
    }

    try {
      const updatedProvider = await ApiClient.ModelProviders.update({
        provider_id: providerId,
        enabled,
      })

      setProviders(prev =>
        prev.map(p => (p.id === providerId ? updatedProvider : p)),
      )
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
            provider.type !== 'llama.cpp' &&
            (!provider.api_key || provider.api_key.trim() === '')
          ) {
            message.error(
              `Cannot enable "${provider.name}" - API key is required`,
            )
          } else if (
            provider.type !== 'llama.cpp' &&
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
          message.error('Failed to update provider')
        }
      } else {
        message.error('Failed to update provider')
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
      const updatedProvider = await ApiClient.ModelProviders.update({
        provider_id: currentProvider.id,
        name: changedValues.name,
      })

      setProviders(prev =>
        prev.map(p => (p.id === currentProvider.id ? updatedProvider : p)),
      )
    } catch (error) {
      console.error('Failed to update provider:', error)
      message.error('Failed to update provider')
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
      const updatedProvider = await ApiClient.ModelProviders.update({
        provider_id: currentProvider.id,
        ...pendingSettings,
      })

      setProviders(prev =>
        prev.map(p => (p.id === currentProvider.id ? updatedProvider : p)),
      )

      // Update form with the new values
      form.setFieldsValue({
        api_key: updatedProvider.api_key,
        base_url: updatedProvider.base_url,
        settings: updatedProvider.settings,
      })

      setHasUnsavedChanges(false)
      setPendingSettings(null)
      message.success('Settings saved successfully')
    } catch (error) {
      console.error('Failed to save settings:', error)
      message.error('Failed to save settings')
    }
  }

  const handleProxySettingsSave = async (proxySettings: any) => {
    if (!currentProvider || !canEditProviders) return

    try {
      const updatedProvider = await ApiClient.ModelProviders.update({
        provider_id: currentProvider.id,
        proxy_settings: proxySettings,
      })

      setProviders(prev =>
        prev.map(p => (p.id === currentProvider.id ? updatedProvider : p)),
      )

      message.success('Proxy settings saved successfully')
    } catch (error) {
      console.error('Failed to save proxy settings:', error)
      message.error('Failed to save proxy settings')
    }
  }

  const handleDeleteProvider = async (providerId: string) => {
    if (!canEditProviders) {
      message.error('You do not have permission to delete providers')
      return
    }

    const provider = providers.find(p => p.id === providerId)
    if (!provider) return

    Modal.confirm({
      title: 'Delete Model Provider',
      content: `Are you sure you want to delete "${provider.name}"? This action cannot be undone.`,
      okText: 'Delete',
      okType: 'danger',
      cancelText: 'Cancel',
      onOk: async () => {
        try {
          await ApiClient.ModelProviders.delete({ provider_id: providerId })

          setProviders(prev => prev.filter(p => p.id !== providerId))
          if (selectedProvider === providerId) {
            const remainingProviders = providers.filter(
              p => p.id !== providerId,
            )
            setSelectedProvider(
              remainingProviders.length > 0 ? remainingProviders[0].id : '',
            )
          }
          message.success('Provider deleted')
        } catch (error: any) {
          console.error('Failed to delete provider:', error)
          if (error.response?.status === 400) {
            message.error(
              `Cannot delete "${provider.name}" - default model providers cannot be deleted`,
            )
          } else if (error.response?.status === 403) {
            message.error(
              'You do not have permission to delete model providers',
            )
          } else if (error.response?.status === 404) {
            message.error('Provider not found')
          } else {
            message.error('Failed to delete provider')
          }
        }
      },
    })
  }

  const handleCloneProvider = async (providerId: string) => {
    if (!canEditProviders) {
      message.error('You do not have permission to clone providers')
      return
    }

    try {
      const clonedProvider = await ApiClient.ModelProviders.clone({
        provider_id: providerId,
      })

      setProviders(prev => [...prev, clonedProvider])
      message.success('Provider cloned successfully')
    } catch (error) {
      console.error('Failed to clone provider:', error)
      message.error('Failed to clone provider')
    }
  }

  const handleAddProvider = async (providerData: any) => {
    try {
      const newProvider = await ApiClient.ModelProviders.create(providerData)

      setProviders(prev => [...prev, newProvider])
      setIsAddModalOpen(false)
      message.success('Provider added successfully')
    } catch (error: any) {
      console.error('Failed to add provider:', error)
      if (error.response?.status === 400) {
        message.error(
          'Failed to add provider - Please check API key and base URL are provided and valid',
        )
      } else {
        message.error('Failed to add provider')
      }
    }
  }

  const handleAddModel = async (modelData: any) => {
    if (!currentProvider) return

    try {
      const newModel = await ApiClient.ModelProviders.addModel({
        provider_id: currentProvider.id,
        ...modelData,
      })

      const updatedProvider = {
        ...currentProvider,
        models: [...(currentProvider.models || []), newModel],
      }

      setProviders(prev =>
        prev.map(p => (p.id === currentProvider.id ? updatedProvider : p)),
      )
      setIsAddModelModalOpen(false)
      message.success('Model added successfully')
    } catch (error) {
      console.error('Failed to add model:', error)
      message.error('Failed to add model')
    }
  }

  const handleEditModel = async (modelData: any) => {
    if (!currentProvider || !selectedModel) return

    try {
      const updatedModel = await ApiClient.Models.update({
        model_id: modelData.id,
        ...modelData,
      })

      const updatedModels = currentProvider.models.map(m =>
        m.id === modelData.id ? updatedModel : m,
      )

      const updatedProvider = {
        ...currentProvider,
        models: updatedModels,
      }

      setProviders(prev =>
        prev.map(p => (p.id === currentProvider.id ? updatedProvider : p)),
      )
      setIsEditModelModalOpen(false)
      message.success('Model updated successfully')
    } catch (error) {
      console.error('Failed to update model:', error)
      message.error('Failed to update model')
    }
  }

  const handleDeleteModel = async (modelId: string) => {
    if (!currentProvider) return

    try {
      await ApiClient.Models.delete({ model_id: modelId })

      const updatedModels = currentProvider.models.filter(m => m.id !== modelId)

      const updatedProvider = {
        ...currentProvider,
        models: updatedModels,
      }

      setProviders(prev =>
        prev.map(p => (p.id === currentProvider.id ? updatedProvider : p)),
      )
      message.success('Model deleted successfully')
    } catch (error) {
      console.error('Failed to delete model:', error)
      message.error('Failed to delete model')
    }
  }

  const handleToggleModel = async (modelId: string, enabled: boolean) => {
    if (!currentProvider) return

    try {
      const updatedModel = await ApiClient.Models.update({
        model_id: modelId,
        enabled,
      })

      const updatedModels = currentProvider.models.map(m =>
        m.id === modelId ? updatedModel : m,
      )

      let updatedProvider = {
        ...currentProvider,
        models: updatedModels,
      }

      // If disabling a model, check if this was the last enabled model
      if (!enabled) {
        const remainingEnabledModels = updatedModels.filter(
          m => m.enabled !== false,
        )

        // If no models are enabled and provider is currently enabled, disable the provider
        if (remainingEnabledModels.length === 0 && currentProvider.enabled) {
          try {
            const disabledProvider = await ApiClient.ModelProviders.update({
              provider_id: currentProvider.id,
              enabled: false,
            })
            updatedProvider = disabledProvider
            message.success(
              `${updatedModel.name} disabled. ${currentProvider.name} provider disabled as no models remain active.`,
            )
          } catch (providerError) {
            console.error('Failed to disable provider:', providerError)
            // Still update the model but show warning about provider
            message.warning(
              `${updatedModel.name} disabled, but failed to disable provider automatically`,
            )
          }
        } else {
          message.success(
            `${updatedModel.name} ${enabled ? 'enabled' : 'disabled'}`,
          )
        }
      } else {
        message.success(
          `${updatedModel.name} ${enabled ? 'enabled' : 'disabled'}`,
        )
      }

      setProviders(prev =>
        prev.map(p => (p.id === currentProvider.id ? updatedProvider : p)),
      )
    } catch (error) {
      console.error('Failed to toggle model:', error)
      message.error('Failed to toggle model')
    }
  }

  const handleStartStopModel = async (modelId: string, isActive: boolean) => {
    if (!currentProvider || currentProvider.type !== 'llama.cpp') return

    try {
      const updatedModel = await ApiClient.Models.update({
        model_id: modelId,
        isActive,
      })

      const updatedModels = currentProvider.models.map(m =>
        m.id === modelId ? updatedModel : m,
      )

      const updatedProvider = {
        ...currentProvider,
        models: updatedModels,
      }

      setProviders(prev =>
        prev.map(p => (p.id === currentProvider.id ? updatedProvider : p)),
      )

      message.success(
        `${updatedModel.name} ${isActive ? 'started' : 'stopped'}`,
      )
    } catch (error) {
      console.error('Failed to start/stop model:', error)
      message.error('Failed to start/stop model')
    }
  }

  const copyToClipboard = (text: string) => {
    if (typeof window !== 'undefined' && window.navigator?.clipboard) {
      window.navigator.clipboard.writeText(text)
      message.success('Copied to clipboard')
    } else {
      message.error('Clipboard not available')
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
        label: 'Clone',
        onClick: () => handleCloneProvider(provider.id),
      })

      actions.push({
        key: 'delete',
        icon: <DeleteOutlined />,
        label: 'Delete',
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

  const ResponsiveConfigItem = ({
    title,
    description,
    children,
  }: {
    title: string
    description: string
    children: React.ReactNode
  }) => (
    <Flex
      justify="space-between"
      align={isMobile ? 'flex-start' : 'center'}
      vertical={isMobile}
      gap={isMobile ? 'small' : 0}
    >
      <div style={{ flex: isMobile ? undefined : 1 }}>
        <Text strong>{title}</Text>
        <div>
          <Text type="secondary">{description}</Text>
        </div>
      </div>
      {children}
    </Flex>
  )

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
          description="No provider selected"
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
                label="Provider Name"
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
        {currentProvider.type !== 'llama.cpp' && (
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
              title="API Configuration"
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
                      placeholder="Insert API Key"
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
                      placeholder="Base URL"
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
          title="Models"
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
          {currentProvider.type === 'llama.cpp' && (
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
                        currentProvider.type === 'llama.cpp' &&
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

        {/* Proxy Settings - For non-Llama.cpp providers */}
        {currentProvider.type !== 'llama.cpp' &&
          currentProvider.proxy_settings && (
            <ModelProviderProxySettingsForm
              providerId={currentProvider.id}
              initialSettings={currentProvider.proxy_settings}
              onSave={handleProxySettingsSave}
              disabled={!canEditProviders}
            />
          )}

        {/* Llama.cpp Specific Settings */}
        {currentProvider.type === 'llama.cpp' && currentProvider.settings && (
          <Form
            layout="vertical"
            initialValues={currentProvider.settings}
            onValuesChange={handleSettingsChange}
          >
            <Card
              title="Configuration"
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
              <Space
                direction="vertical"
                size="middle"
                style={{ width: '100%' }}
              >
                <ResponsiveConfigItem
                  title="Auto-Unload Old Models"
                  description="Automatically unloads models that are not in use to free up memory. Ensure only one model is loaded at a time."
                >
                  <Form.Item
                    name="autoUnloadOldModels"
                    valuePropName="checked"
                    style={{ margin: 0 }}
                  >
                    <Switch disabled={!canEditProviders} />
                  </Form.Item>
                </ResponsiveConfigItem>

                <Divider style={{ margin: 0 }} />

                <ResponsiveConfigItem
                  title="Context Shift"
                  description="Automatically shifts the context window when the model is unable to process the entire prompt, ensuring that the most relevant information is always included."
                >
                  <Form.Item
                    name="contextShift"
                    valuePropName="checked"
                    style={{ margin: 0 }}
                  >
                    <Switch disabled={!canEditProviders} />
                  </Form.Item>
                </ResponsiveConfigItem>

                <Divider style={{ margin: 0 }} />

                <ResponsiveConfigItem
                  title="Continuous Batching"
                  description="Allows processing prompts in parallel with text generation, which usually improves performance."
                >
                  <Form.Item
                    name="continuousBatching"
                    valuePropName="checked"
                    style={{ margin: 0 }}
                  >
                    <Switch disabled={!canEditProviders} />
                  </Form.Item>
                </ResponsiveConfigItem>

                <Divider style={{ margin: 0 }} />

                <ResponsiveConfigItem
                  title="Parallel Operations"
                  description="Number of prompts that can be processed simultaneously by the model."
                >
                  <Form.Item
                    name="parallelOperations"
                    style={{ margin: 0, width: isMobile ? '100%' : 100 }}
                  >
                    <InputNumber
                      min={1}
                      max={16}
                      style={{ width: '100%' }}
                      disabled={!canEditProviders}
                    />
                  </Form.Item>
                </ResponsiveConfigItem>

                <Divider style={{ margin: 0 }} />

                <ResponsiveConfigItem
                  title="CPU Threads"
                  description="Number of CPU cores used for model processing when running without GPU."
                >
                  <Form.Item
                    name="cpuThreads"
                    style={{ margin: 0, width: isMobile ? '100%' : 100 }}
                  >
                    <InputNumber
                      placeholder="-1 (auto)"
                      style={{ width: '100%' }}
                      disabled={!canEditProviders}
                    />
                  </Form.Item>
                </ResponsiveConfigItem>

                <Divider style={{ margin: 0 }} />

                <ResponsiveConfigItem
                  title="Threads (Batch)"
                  description="Number of threads for batch and prompt processing (default: same as Threads)."
                >
                  <Form.Item
                    name="threadsBatch"
                    style={{ margin: 0, width: isMobile ? '100%' : 100 }}
                  >
                    <InputNumber
                      placeholder="-1 (same as Threads)"
                      style={{ width: '100%' }}
                      disabled={!canEditProviders}
                    />
                  </Form.Item>
                </ResponsiveConfigItem>

                <Divider style={{ margin: 0 }} />

                <ResponsiveConfigItem
                  title="Flash Attention"
                  description="Optimizes memory usage and speeds up model inference using an efficient attention implementation."
                >
                  <Form.Item
                    name="flashAttention"
                    valuePropName="checked"
                    style={{ margin: 0 }}
                  >
                    <Switch disabled={!canEditProviders} />
                  </Form.Item>
                </ResponsiveConfigItem>

                <Divider style={{ margin: 0 }} />

                <ResponsiveConfigItem
                  title="Caching"
                  description="Stores recent prompts and responses to improve speed when similar questions are asked."
                >
                  <Form.Item
                    name="caching"
                    valuePropName="checked"
                    style={{ margin: 0 }}
                  >
                    <Switch disabled={!canEditProviders} />
                  </Form.Item>
                </ResponsiveConfigItem>

                <Divider style={{ margin: 0 }} />

                <ResponsiveConfigItem
                  title="KV Cache Type"
                  description="Controls memory usage and precision trade-off."
                >
                  <Form.Item
                    name="kvCacheType"
                    style={{ margin: 0, width: isMobile ? '100%' : 100 }}
                  >
                    <Select
                      style={{ width: '100%' }}
                      disabled={!canEditProviders}
                      options={[
                        { value: 'q8_0', label: 'q8_0' },
                        { value: 'q4_0', label: 'q4_0' },
                        { value: 'q4_1', label: 'q4_1' },
                        { value: 'q5_0', label: 'q5_0' },
                        { value: 'q5_1', label: 'q5_1' },
                      ]}
                    />
                  </Form.Item>
                </ResponsiveConfigItem>

                <Divider style={{ margin: 0 }} />

                <ResponsiveConfigItem
                  title="mmap"
                  description="Loads model files more efficiently by mapping them to memory, reducing RAM usage."
                >
                  <Form.Item
                    name="mmap"
                    valuePropName="checked"
                    style={{ margin: 0 }}
                  >
                    <Switch disabled={!canEditProviders} />
                  </Form.Item>
                </ResponsiveConfigItem>

                <Divider style={{ margin: 0 }} />

                <div>
                  <Text strong>Hugging Face Access Token</Text>
                  <div>
                    <Text type="secondary">
                      Access tokens programmatically authenticate your identity
                      to the Hugging Face Hub, allowing applications to perform
                      specific actions specified by the scope of permissions
                      granted.
                    </Text>
                  </div>
                  <Form.Item
                    name="huggingFaceAccessToken"
                    style={{ marginTop: 8, marginBottom: 0 }}
                  >
                    <Input.Password
                      placeholder="hf_*****************************"
                      style={{ width: '100%' }}
                      disabled={!canEditProviders}
                    />
                  </Form.Item>
                </div>
              </Space>
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
