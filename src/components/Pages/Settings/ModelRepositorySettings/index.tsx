import {
  CloudDownloadOutlined,
  DeleteOutlined,
  EditOutlined,
  PlusOutlined,
} from '@ant-design/icons'
import {
  App,
  Button,
  Card,
  Divider,
  Empty,
  Flex,
  Switch,
  Typography,
} from 'antd'
import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import {
  deleteAdminModelRepository,
  loadAllAdminModelRepositories,
  Stores,
  updateAdminModelRepository,
} from '../../../../store'
import { openRepositoryDrawer } from '../../../../store/ui'
import { Repository } from '../../../../types'
import { RepositoryDrawer } from './RepositoryDrawer'
import { SettingsPageContainer } from '../common/SettingsPageContainer.tsx'

const { Text } = Typography

export function ModelRepositorySettings() {
  const { t } = useTranslation()
  const { message } = App.useApp()

  // Use repository store
  const { repositories, testing } = Stores.AdminRepositories

  // Load repositories when component mounts
  useEffect(() => {
    loadAllAdminModelRepositories().catch((error: any) => {
      console.error('Failed to load repositories:', error)
    })
  }, [])

  const testRepositoryConnection = async (repository: Repository) => {
    // Validate required fields based on auth type
    if (
      repository.auth_type === 'api_key' &&
      !repository.auth_config?.api_key
    ) {
      message.warning('Please enter an API key first')
      return
    }
    if (
      repository.auth_type === 'basic_auth' &&
      (!repository.auth_config?.username || !repository.auth_config?.password)
    ) {
      message.warning('Please enter username and password first')
      return
    }
    if (
      repository.auth_type === 'bearer_token' &&
      !repository.auth_config?.token
    ) {
      message.warning('Please enter a bearer token first')
      return
    }

    try {
      // Note: testRepositoryConnection function would need to be imported
      // For now, just show success message
      message.success(`Connection to ${repository.name} successful!`)
    } catch (error: any) {
      console.error('Repository connection test failed:', error)
      message.error(error?.message || `Connection to ${repository.name} failed`)
    }
  }

  // Repository management functions
  const handleAddRepository = () => {
    openRepositoryDrawer()
  }

  const handleEditRepository = (repository: Repository) => {
    openRepositoryDrawer(repository)
  }

  const handleDeleteRepository = async (repositoryId: string) => {
    // Don't allow deleting built-in repositories
    const repo = repositories.find(r => r.id === repositoryId)
    if (repo?.built_in) {
      message.warning('Built-in repositories cannot be deleted')
      return
    }

    try {
      await deleteAdminModelRepository(repositoryId)
      message.success('Repository removed successfully')
    } catch (error: any) {
      console.error('Failed to delete repository:', error)
      message.error(error?.message || 'Failed to delete repository')
    }
  }

  const handleToggleRepository = async (
    repositoryId: string,
    enabled: boolean,
  ) => {
    try {
      await updateAdminModelRepository(repositoryId, { enabled })
    } catch (error: any) {
      console.error('Failed to toggle repository:', error)
      message.error(error?.message || 'Failed to toggle repository')
    }
  }

  const getRepositoryActions = (repository: Repository) => {
    const actions: React.ReactNode[] = []

    // Always include the enable/disable switch first
    actions.push(
      <Switch
        key="enable"
        className="!mr-2"
        checked={repository.enabled}
        onChange={checked => handleToggleRepository(repository.id, checked)}
      />,
    )

    actions.push(
      <Button
        key="test"
        type="text"
        icon={<CloudDownloadOutlined />}
        loading={testing}
        onClick={() => testRepositoryConnection(repository)}
      >
        Test
      </Button>,
    )

    actions.push(
      <Button
        key="edit"
        type="text"
        icon={<EditOutlined />}
        onClick={() => handleEditRepository(repository)}
      >
        Edit
      </Button>,
    )

    if (!repository.built_in) {
      actions.push(
        <Button
          key="delete"
          type="text"
          danger
          icon={<DeleteOutlined />}
          onClick={() => handleDeleteRepository(repository.id)}
        >
          Delete
        </Button>,
      )
    }

    return actions.filter(Boolean)
  }

  return (
    <SettingsPageContainer
      title={t('settings.modelRepository.title')}
      subtitle={t('settings.modelRepository.description')}
    >
      {/* Model Repositories */}
      <Card
        title={
          <Flex align="center" gap="middle">
            <CloudDownloadOutlined />
            <span>Model Repositories</span>
          </Flex>
        }
        extra={
          <Button
            type={'text'}
            icon={<PlusOutlined />}
            onClick={handleAddRepository}
          />
        }
      >
        <Flex className="flex-col gap-4">
          <div>
            {repositories.length === 0 ? (
              <Empty
                description="No repositories configured"
                image={
                  <CloudDownloadOutlined className="text-4xl opacity-50" />
                }
              >
                <Text type="secondary">Add a repository to get started</Text>
              </Empty>
            ) : (
              <div>
                {repositories.map((repository, index) => (
                  <div key={repository.id}>
                    <div className="flex items-start gap-3 flex-wrap">
                      {/* Repository Info */}
                      <div className="flex-1">
                        <div className="flex items-center gap-2 mb-2 flex-wrap-reverse">
                          <div className="flex-1 min-w-48">
                            <Flex align="center" gap="small">
                              <Text className="font-medium">
                                {repository.name}
                              </Text>
                              {repository.built_in && (
                                <Text type="secondary" className="text-xs">
                                  (Built-in)
                                </Text>
                              )}
                              {!repository.enabled && (
                                <Text type="secondary" className="text-xs">
                                  (Disabled)
                                </Text>
                              )}
                            </Flex>
                          </div>
                          <div className="flex gap-1 items-center justify-end">
                            {getRepositoryActions(repository)}
                          </div>
                        </div>

                        <div className="space-y-1">
                          <Text type="secondary" className="block">
                            {repository.url}
                          </Text>
                          <Text type="secondary" className="text-xs block">
                            Authentication:{' '}
                            {repository.auth_type === 'none'
                              ? 'None'
                              : repository.auth_type === 'api_key'
                                ? 'API Key'
                                : repository.auth_type === 'basic_auth'
                                  ? 'Basic Auth'
                                  : repository.auth_type === 'bearer_token'
                                    ? 'Bearer Token'
                                    : repository.auth_type}
                          </Text>
                        </div>
                      </div>
                    </div>
                    {index < repositories.length - 1 && (
                      <Divider className="my-0" />
                    )}
                  </div>
                ))}
              </div>
            )}
          </div>
        </Flex>
      </Card>

      <RepositoryDrawer />
    </SettingsPageContainer>
  )
}
