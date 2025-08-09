import {
  CopyOutlined,
  DeleteOutlined,
  DownOutlined,
  MenuOutlined,
  PlusOutlined,
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
  clearRAGProvidersError,
  cloneExistingRAGProvider,
  deleteRAGProvider,
  loadAllRAGProviders,
  openAddRAGProviderDrawer,
  closeAddRAGProviderDrawer,
  closeAddRAGDatabaseDrawer,
  closeAddRAGDatabaseDownloadDrawer,
  closeEditRAGDatabaseDrawer,
  useAddRAGProviderDrawerStore,
  useAddRAGDatabaseDrawerStore,
  useAddRAGDatabaseDownloadDrawerStore,
  useEditRAGDatabaseDrawerStore,
  Stores,
} from '../../../../store'
import { RAGProvider, RAGProviderType } from '../../../../types/api/ragProvider'
import { LocalRAGProviderSettings } from './LocalRAGProviderSettings'
import { RemoteRAGProviderSettings } from './RemoteRAGProviderSettings'
import { SettingsPageContainer } from '../SettingsPageContainer'
import { AddRAGProviderDrawer } from './AddRAGProviderDrawer'
import { AddRAGDatabaseDrawer } from './AddRAGDatabaseDrawer'
import { AddRAGDatabaseDownloadDrawer } from './AddRAGDatabaseDownloadDrawer'
import { EditRAGDatabaseDrawer } from './EditRAGDatabaseDrawer'

const { Title, Text } = Typography
const { Sider, Content } = Layout

const RAG_PROVIDER_ICONS: Record<RAGProviderType, string> = {
  local: '🏠',
  lightrag: '🔍',
  ragstack: '📚',
  chroma: '🌈',
  weaviate: '🕷️',
  pinecone: '🌲',
  custom: '🔧',
}

export function RAGProvidersSettings() {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const { hasPermission } = usePermissions()
  const { provider_id } = useParams<{ provider_id?: string }>()
  const navigate = useNavigate()

  // RAG providers store
  const { providers, loading, error } = Stores.AdminRAGProviders

  const [isMobile, setIsMobile] = useState(false)

  // UI drawer states
  const addRAGProviderDrawer = useAddRAGProviderDrawerStore()
  const addRAGDatabaseDrawer = useAddRAGDatabaseDrawerStore()
  const addRAGDatabaseDownloadDrawer = useAddRAGDatabaseDownloadDrawerStore()
  const editRAGDatabaseDrawer = useEditRAGDatabaseDrawerStore()

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
          You do not have permission to view RAG provider settings.
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
    loadAllRAGProviders()
  }, [])

  // Show errors
  useEffect(() => {
    if (error) {
      message.error(error)
      clearRAGProvidersError()
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
          navigate(`/settings/rag-providers/${providers[0].id}`, {
            replace: true,
          })
        }
      } else {
        // No URL parameter, navigate to first provider
        navigate(`/settings/rag-providers/${providers[0].id}`, {
          replace: true,
        })
      }
    }
  }, [providers, provider_id, navigate])

  const handleDeleteProvider = async (providerId: string) => {
    if (!canEditProviders) {
      message.error('No permission to delete RAG providers')
      return
    }

    const provider = providers.find(p => p.id === providerId)
    if (!provider) return

    Modal.confirm({
      title: 'Delete RAG Provider',
      content: `Are you sure you want to delete "${provider.name}"? This action cannot be undone.`,
      okText: 'Delete',
      okType: 'danger',
      cancelText: 'Cancel',
      onOk: async () => {
        try {
          await deleteRAGProvider(providerId)
          if (provider_id === providerId) {
            const remainingProviders = providers.filter(
              p => p.id !== providerId,
            )
            if (remainingProviders.length > 0) {
              navigate(`/settings/rag-providers/${remainingProviders[0].id}`, {
                replace: true,
              })
            } else {
              navigate('/settings/rag-providers', { replace: true })
            }
          }
          message.success('RAG provider deleted successfully')
        } catch (error: any) {
          console.error('Failed to delete RAG provider:', error)
          // Error is handled by the store
        }
      },
    })
  }

  const handleCloneProvider = async (providerId: string) => {
    if (!canEditProviders) {
      message.error('No permission to clone RAG providers')
      return
    }

    try {
      await cloneExistingRAGProvider(providerId)
      message.success('RAG provider cloned successfully')
    } catch (error) {
      console.error('Failed to clone RAG provider:', error)
      // Error is handled by the store
    }
  }

  const getProviderActions = (provider: RAGProvider) => {
    const actions: any[] = []

    if (canEditProviders) {
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
        <span className={'text-lg'}>{RAG_PROVIDER_ICONS[provider.type]}</span>
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
      key: 'add-rag-provider',
      //@ts-ignore
      icon: <PlusOutlined />,
      label: <Typography.Text>Add RAG Provider</Typography.Text>,
    })
  }

  const ProviderMenu = () => (
    <Menu
      selectedKeys={provider_id ? [provider_id] : []}
      items={menuItems}
      onClick={({ key }) => {
        if (key === 'add-rag-provider') {
          openAddRAGProviderDrawer()
        } else {
          navigate(`/settings/rag-providers/${key}`)
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
          description="No RAG provider selected"
          image={Empty.PRESENTED_IMAGE_SIMPLE}
        />
      )
    }

    // Render appropriate provider settings component based on type
    if (currentProvider.type === 'local') {
      return <LocalRAGProviderSettings />
    }

    return <RemoteRAGProviderSettings />
  }

  return (
    <SettingsPageContainer title="RAG Providers">
      <div className={'flex w-full'}>
        {/* Desktop Sidebar */}
        {!isMobile && (
          <Sider
            width={200}
            theme="light"
            style={{ backgroundColor: 'transparent' }}
          >
            <div>
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
                <Dropdown
                  menu={{
                    items: menuItems,
                    onClick: ({ key }) => {
                      if (key === 'add-rag-provider') {
                        openAddRAGProviderDrawer()
                      } else {
                        navigate(`/settings/rag-providers/${key}`)
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
                            ? RAG_PROVIDER_ICONS[currentProvider.type]
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
        <AddRAGProviderDrawer
          open={addRAGProviderDrawer.open}
          onClose={closeAddRAGProviderDrawer}
        />
        <AddRAGDatabaseDrawer
          open={addRAGDatabaseDrawer.open}
          onClose={closeAddRAGDatabaseDrawer}
          providerId={addRAGDatabaseDrawer.providerId}
        />
        <AddRAGDatabaseDownloadDrawer
          open={addRAGDatabaseDownloadDrawer.open}
          onClose={closeAddRAGDatabaseDownloadDrawer}
          providerId={addRAGDatabaseDownloadDrawer.providerId}
        />
        <EditRAGDatabaseDrawer
          open={editRAGDatabaseDrawer.open}
          onClose={closeEditRAGDatabaseDrawer}
          database={editRAGDatabaseDrawer.database}
        />
      </div>
    </SettingsPageContainer>
  )
}
