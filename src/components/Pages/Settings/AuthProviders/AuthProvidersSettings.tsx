import {
  DeleteOutlined,
  EditOutlined,
  ExperimentOutlined,
  PlusOutlined,
  SafetyOutlined,
} from '@ant-design/icons'
import {
  App,
  Button,
  Card,
  Descriptions,
  Divider,
  Empty,
  Flex,
  Popconfirm,
  Spin,
  Tag,
  Typography,
} from 'antd'
import { useEffect, useState } from 'react'
import { isTauriView } from '../../../../api/core.ts'
import {
  clearProvidersError,
  deleteAuthProvider,
  loadAuthProviders,
  Stores,
  testAuthProviderConnection,
  toggleAuthProviderEnabled,
} from '../../../../store'
import type { AuthProvider, TestConnectionResult } from '../../../../types'
import { SettingsPageContainer } from '../common/SettingsPageContainer.tsx'
import { TestConnectionModal } from './TestConnectionModal.tsx'
import { AuthProviderModal } from './AuthProviderModal.tsx'

const { Title, Text } = Typography

export function AuthProvidersSettings() {
  const { message } = App.useApp()

  const { providers, loading, error, testing } = Stores.AdminAuthProviders

  const [selectedProvider, setSelectedProvider] = useState<AuthProvider | null>(
    null,
  )
  const [createModalVisible, setCreateModalVisible] = useState(false)
  const [editModalVisible, setEditModalVisible] = useState(false)
  const [testModalVisible, setTestModalVisible] = useState(false)
  const [testResult, setTestResult] = useState<TestConnectionResult | null>(
    null,
  )

  // Show errors
  useEffect(() => {
    if (error) {
      message.error(error)
      clearProvidersError()
    }
  }, [error, message])

  // Load providers on mount
  useEffect(() => {
    if (!isTauriView) {
      loadAuthProviders().catch(console.error)
    }
  }, [])

  const handleDeleteProvider = async (providerId: string) => {
    try {
      await deleteAuthProvider(providerId)
      message.success('Auth provider deleted successfully')
    } catch (error) {
      console.error('Failed to delete auth provider:', error)
    }
  }

  const handleToggleEnabled = async (providerId: string, enabled: boolean) => {
    try {
      await toggleAuthProviderEnabled(providerId, enabled)
      message.success(
        `Provider ${enabled ? 'enabled' : 'disabled'} successfully`,
      )
    } catch (error) {
      console.error('Failed to toggle provider:', error)
    }
  }

  const handleTestConnection = async (provider: AuthProvider) => {
    setSelectedProvider(provider)
    setTestModalVisible(true)
    setTestResult(null)

    try {
      const result = await testAuthProviderConnection(provider.id)
      setTestResult(result)
      if (result.success) {
        message.success('Connection test successful')
      } else {
        message.error('Connection test failed')
      }
    } catch (error) {
      console.error('Connection test failed:', error)
      setTestResult({
        success: false,
        message:
          error instanceof Error ? error.message : 'Connection test failed',
      })
    }
  }

  const openEditModal = (provider: AuthProvider) => {
    setSelectedProvider(provider)
    setEditModalVisible(true)
  }

  const handleModalSuccess = () => {
    // Reload providers after create/edit
    loadAuthProviders().catch(console.error)
  }

  const getProviderTypeLabel = (providerType: string) => {
    switch (providerType) {
      case 'oauth2':
        return 'OAuth 2.0'
      case 'oidc':
        return 'OpenID Connect'
      case 'saml':
        return 'SAML'
      case 'ldap':
        return 'LDAP'
      default:
        return providerType
    }
  }

  const getProviderActions = (provider: AuthProvider) => {
    const actions: React.ReactNode[] = []
    const isLocalProvider = provider.provider_type === 'local'

    // Enable/Disable button (not for local provider)
    if (!isLocalProvider) {
      actions.push(
        <Button
          key="toggle"
          type="text"
          onClick={() => handleToggleEnabled(provider.id, !provider.enabled)}
        >
          {provider.enabled ? 'Disable' : 'Enable'}
        </Button>,
      )
    }

    // Test button (not for local provider)
    if (!isLocalProvider) {
      actions.push(
        <Button
          key="test"
          type="text"
          icon={<ExperimentOutlined />}
          onClick={() => handleTestConnection(provider)}
          loading={testing[provider.id]}
        >
          Test
        </Button>,
      )
    }

    actions.push(
      <Button
        key="edit"
        type="text"
        icon={<EditOutlined />}
        onClick={() => openEditModal(provider)}
      >
        Edit
      </Button>,
    )

    // Delete button (not for local provider)
    if (!isLocalProvider) {
      actions.push(
        <Popconfirm
          key="delete"
          title="Are you sure you want to delete this provider?"
          onConfirm={() => handleDeleteProvider(provider.id)}
          okText="Yes"
          cancelText="No"
        >
          <Button type="text" danger icon={<DeleteOutlined />}>
            Delete
          </Button>
        </Popconfirm>,
      )
    }

    return actions.filter(Boolean)
  }

  if (isTauriView) {
    return (
      <Card>
        <div className="text-center">
          <Title level={4}>Authentication Providers</Title>
          <Text type="secondary">
            Authentication provider management is disabled in desktop mode
          </Text>
        </div>
      </Card>
    )
  }

  return (
    <SettingsPageContainer title="Authentication Providers">
      <div>
        <Card
          title="Authentication Providers"
          extra={
            <Button
              type="primary"
              icon={<PlusOutlined />}
              onClick={() => setCreateModalVisible(true)}
            >
              Add Provider
            </Button>
          }
        >
          {loading ? (
            <div className="flex justify-center py-8">
              <Spin size="large" />
            </div>
          ) : providers.length === 0 ? (
            <div>
              <Empty description="No authentication providers configured" />
            </div>
          ) : (
            <div>
              {providers.map((provider, index) => (
                <div key={provider.id}>
                  <div className="flex items-start gap-3 flex-wrap">
                    {/* Provider Info */}
                    <div className="flex-1">
                      <div className="flex items-center gap-2 mb-2 flex-wrap">
                        <div className={'flex-1 min-w-48'}>
                          <Flex className="gap-2 items-center">
                            <SafetyOutlined />
                            <Text className="font-medium">{provider.name}</Text>
                            <Tag color="blue">
                              {getProviderTypeLabel(provider.provider_type)}
                            </Tag>
                            <Tag color={provider.enabled ? 'green' : 'default'}>
                              {provider.enabled ? 'Enabled' : 'Disabled'}
                            </Tag>
                          </Flex>
                        </div>
                        <div className={'flex gap-1 items-center justify-end'}>
                          {getProviderActions(provider)}
                        </div>
                      </div>

                      <Descriptions
                        size="small"
                        column={{ xs: 1, sm: 2, md: 3 }}
                        colon={false}
                        labelStyle={{ fontSize: '12px', color: '#8c8c8c' }}
                        contentStyle={{ fontSize: '12px' }}
                      >
                        <Descriptions.Item label="Priority">
                          {provider.priority}
                        </Descriptions.Item>
                        <Descriptions.Item label="Created">
                          {new Date(provider.created_at).toLocaleDateString()}
                        </Descriptions.Item>
                        <Descriptions.Item label="Updated">
                          {new Date(provider.updated_at).toLocaleDateString()}
                        </Descriptions.Item>
                      </Descriptions>
                    </div>
                  </div>
                  {index < providers.length - 1 && <Divider className="my-0" />}
                </div>
              ))}
            </div>
          )}
        </Card>

        {/* Test Connection Modal */}
        <TestConnectionModal
          open={testModalVisible}
          onClose={() => {
            setTestModalVisible(false)
            setSelectedProvider(null)
            setTestResult(null)
          }}
          provider={selectedProvider}
          testResult={testResult}
          testing={
            selectedProvider ? testing[selectedProvider.id] || false : false
          }
        />

        {/* Create Provider Modal */}
        <AuthProviderModal
          open={createModalVisible}
          onClose={() => setCreateModalVisible(false)}
          onSuccess={handleModalSuccess}
        />

        {/* Edit Provider Modal */}
        <AuthProviderModal
          open={editModalVisible}
          onClose={() => {
            setEditModalVisible(false)
            setSelectedProvider(null)
          }}
          onSuccess={handleModalSuccess}
          provider={selectedProvider}
        />
      </div>
    </SettingsPageContainer>
  )
}
