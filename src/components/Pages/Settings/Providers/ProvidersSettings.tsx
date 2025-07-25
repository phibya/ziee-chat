import {
  CopyOutlined,
  DeleteOutlined,
  DownOutlined,
  MenuOutlined,
  PlusOutlined,
  SettingOutlined,
} from '@ant-design/icons'
import {
  App,
  Button,
  Dropdown,
  Empty,
  Flex,
  Layout,
  Menu,
  Modal,
  Spin,
  Typography,
} from 'antd'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useNavigate, useParams } from 'react-router-dom'
import { isDesktopApp } from '../../../../api/core'
import { Permission, usePermissions } from '../../../../permissions'
import {
  clearProvidersError,
  cloneExistingProvider,
  deleteModelProvider,
  loadAllModelProviders,
  openAddProviderDrawer,
  Stores,
} from '../../../../store'
import { Provider, ProviderType } from '../../../../types/api/provider'
import { AddLocalModelDownloadDrawer } from './AddLocalModelDownloadDrawer.tsx'
import { AddLocalModelUploadDrawer } from './AddLocalModelUploadDrawer.tsx'
import { AddProviderDrawer } from './AddProviderDrawer.tsx'
import { AddRemoteModelDrawer } from './AddRemoteModelDrawer.tsx'
import { EditLocalModelDrawer } from './EditLocalModelDrawer.tsx'
import { EditRemoteModelDrawer } from './EditRemoteModelDrawer.tsx'
import { LocalProviderSettings } from './LocalProviderSettings'
import { RemoteProviderSettings } from './RemoteProviderSettings'

const { Title, Text } = Typography
const { Sider, Content } = Layout

const PROVIDER_ICONS: Record<ProviderType, string> = {
  local: '🕯',
  openai: '🤖',
  anthropic: '🤖',
  groq: '⚡',
  gemini: '💎',
  mistral: '🌊',
  custom: '🔧',
}

export function ProvidersSettings() {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const { hasPermission } = usePermissions()
  const { provider_id } = useParams<{ provider_id?: string }>()
  const navigate = useNavigate()

  // Model providers store
  const { providers, loading, error } = Stores.Providers

  const [isMobile, setIsMobile] = useState(false)

  // Check permissions for web app
  const canEditProviders =
    isDesktopApp || hasPermission(Permission.config.providers.edit)
  const canViewProviders =
    isDesktopApp || hasPermission(Permission.config.providers.read)

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

  const currentProvider = providers.find(p => p.id === provider_id)

  useEffect(() => {
    loadAllModelProviders()
  }, [])

  // Show errors
  useEffect(() => {
    if (error) {
      message.error(error)
      clearProvidersError()
    }
  }, [error, message])

  // Handle URL parameter and provider selection
  useEffect(() => {
    if (providers.length > 0) {
      if (provider_id) {
        // If URL has provider_id, check if it's valid
        const providerExists = providers.find(p => p.id === provider_id)
        if (!providerExists) {
          // Provider doesn't exist, redirect to first provider
          navigate(`/settings/providers/${providers[0].id}`, {
            replace: true,
          })
        }
      } else {
        // No URL parameter, navigate to first provider
        navigate(`/settings/providers/${providers[0].id}`, {
          replace: true,
        })
      }
    }
  }, [providers, provider_id, navigate])

  const handleDeleteProvider = async (providerId: string) => {
    if (!canEditProviders) {
      message.error(t('providers.noPermissionDelete'))
      return
    }

    const provider = providers.find(p => p.id === providerId)
    if (!provider) return

    Modal.confirm({
      title: t('providers.deleteProvider'),
      content: `Are you sure you want to delete "${provider.name}"? This action cannot be undone.`,
      okText: 'Delete',
      okType: 'danger',
      cancelText: 'Cancel',
      onOk: async () => {
        try {
          await deleteModelProvider(providerId)
          if (provider_id === providerId) {
            const remainingProviders = providers.filter(
              p => p.id !== providerId,
            )
            if (remainingProviders.length > 0) {
              navigate(`/settings/providers/${remainingProviders[0].id}`, {
                replace: true,
              })
            } else {
              navigate('/settings/providers', { replace: true })
            }
          }
          message.success(t('providers.providerDeleted'))
        } catch (error: any) {
          console.error('Failed to delete provider:', error)
          // Error is handled by the store
        }
      },
    })
  }

  const handleCloneProvider = async (providerId: string) => {
    if (!canEditProviders) {
      message.error(t('providers.noPermissionClone'))
      return
    }

    try {
      await cloneExistingProvider(providerId)
      message.success(t('providers.providerCloned'))
    } catch (error) {
      console.error('Failed to clone provider:', error)
      // Error is handled by the store
    }
  }

  const getProviderActions = (provider: Provider) => {
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
        disabled: provider.built_in,
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
      selectedKeys={provider_id ? [provider_id] : []}
      items={menuItems}
      onClick={({ key }) => {
        if (key === 'add-provider') {
          openAddProviderDrawer()
        } else {
          navigate(`/settings/providers/${key}`)
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
          description={t('providers.noProviderSelected')}
          image={Empty.PRESENTED_IMAGE_SIMPLE}
        />
      )
    }

    // Render appropriate provider settings component based on type
    if (currentProvider.type === 'local') {
      return <LocalProviderSettings />
    }

    return <RemoteProviderSettings />
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
            <Title level={3}>Providers</Title>
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
                Providers
              </Title>
              <Dropdown
                menu={{
                  items: menuItems,
                  onClick: ({ key }) => {
                    if (key === 'add-provider') {
                      openAddProviderDrawer()
                    } else {
                      navigate(`/settings/providers/${key}`)
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
      <AddProviderDrawer />

      <AddLocalModelUploadDrawer />
      <AddLocalModelDownloadDrawer />
      <AddRemoteModelDrawer />

      <EditLocalModelDrawer />
      <EditRemoteModelDrawer />
    </Layout>
  )
}
