import {
  CloudDownloadOutlined,
  DeleteOutlined,
  EditOutlined,
  EyeInvisibleOutlined,
  EyeTwoTone,
  PlusOutlined,
} from '@ant-design/icons'
import {
  App,
  Button,
  Card,
  Flex,
  Form,
  Input,
  List,
  Modal,
  Select,
  Space,
  Switch,
  Typography,
} from 'antd'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useShallow } from 'zustand/react/shallow'
import { isDesktopApp } from '../../../api/core'
import { Permission, usePermissions } from '../../../permissions'
import { useRepositoriesStore } from '../../../store/repositories'
import { Repository } from '../../../types/api/repository'

const { Title, Text } = Typography

export function ModelRepositorySettings() {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const { hasPermission } = usePermissions()
  const [repositoryForm] = Form.useForm()
  const [isRepositoryModalOpen, setIsRepositoryModalOpen] = useState(false)
  const [editingRepository, setEditingRepository] = useState<Repository | null>(
    null,
  )

  // Use repository store
  const {
    repositories,
    creating,
    updating,
    testing,
    loadRepositories,
    createRepository,
    updateRepository,
    deleteRepository,
    testConnection,
  } = useRepositoriesStore(
    useShallow(state => ({
      repositories: state.repositories,
      creating: state.creating,
      updating: state.updating,
      testing: state.testing,
      loadRepositories: state.loadRepositories,
      createRepository: state.createRepository,
      updateRepository: state.updateRepository,
      deleteRepository: state.deleteRepository,
      testConnection: state.testConnection,
    })),
  )

  // Check permissions
  const canViewRepositories =
    isDesktopApp || hasPermission(Permission.config.repositories.read)
  const canEditRepositories =
    isDesktopApp || hasPermission(Permission.config.repositories.edit)

  // If user doesn't have view permissions, don't render the component
  if (!canViewRepositories) {
    return (
      <div className="max-w-4xl">
        <div style={{ padding: '24px', textAlign: 'center' }}>
          <Title level={3}>Access Denied</Title>
          <Text type="secondary">
            You do not have permission to view model repository settings.
          </Text>
        </div>
      </div>
    )
  }

  // Load repositories when component mounts
  useEffect(() => {
    loadRepositories().catch(error => {
      console.error('Failed to load repositories:', error)
    })
  }, [loadRepositories])

  // Update repository form when editing
  useEffect(() => {
    if (editingRepository && isRepositoryModalOpen) {
      repositoryForm.setFieldsValue({
        name: editingRepository.name,
        url: editingRepository.url,
        auth_type: editingRepository.auth_type,
        api_key: editingRepository.auth_config?.api_key,
        username: editingRepository.auth_config?.username,
        password: editingRepository.auth_config?.password,
        token: editingRepository.auth_config?.token,
        enabled: editingRepository.enabled,
      })
    } else if (!editingRepository && isRepositoryModalOpen) {
      repositoryForm.setFieldsValue({
        auth_type: 'none',
        enabled: true,
      })
    }
  }, [editingRepository, isRepositoryModalOpen, repositoryForm])

  const testRepositoryConnection = async (repository: Repository) => {
    if (!canEditRepositories) {
      message.error('You do not have permission to test repository connections')
      return
    }

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
      const result = await testConnection({
        name: repository.name,
        url: repository.url,
        auth_type: repository.auth_type,
        auth_config: repository.auth_config,
      })

      if (result.success) {
        message.success(`Connection to ${repository.name} successful!`)
      } else {
        message.error(result.message || `Connection to ${repository.name} failed`)
      }
    } catch (error: any) {
      console.error('Repository connection test failed:', error)
      message.error(error?.message || `Connection to ${repository.name} failed`)
    }
  }

  const testRepositoryFromForm = async () => {
    if (!canEditRepositories) {
      message.error('You do not have permission to test repository connections')
      return
    }

    const values = repositoryForm.getFieldsValue()

    // Validate required fields
    if (!values.name) {
      message.warning('Please enter a repository name first')
      return
    }
    if (!values.url) {
      message.warning('Please enter a repository URL first')
      return
    }

    // Validate auth fields based on type
    if (values.auth_type === 'api_key' && !values.api_key) {
      message.warning('Please enter an API key first')
      return
    }
    if (
      values.auth_type === 'basic_auth' &&
      (!values.username || !values.password)
    ) {
      message.warning('Please enter username and password first')
      return
    }
    if (values.auth_type === 'bearer_token' && !values.token) {
      message.warning('Please enter a bearer token first')
      return
    }

    try {
      const result = await testConnection({
        name: values.name,
        url: values.url,
        auth_type: values.auth_type,
        auth_config: {
          api_key: values.api_key,
          username: values.username,
          password: values.password,
          token: values.token,
        },
      })

      if (result.success) {
        message.success(`Connection to ${values.name} successful!`)
      } else {
        message.error(result.message || `Connection to ${values.name} failed`)
      }
    } catch (error: any) {
      console.error('Repository connection test failed:', error)
      message.error(error?.message || `Connection to ${values.name} failed`)
    }
  }

  // Repository management functions
  const handleAddRepository = () => {
    setEditingRepository(null)
    setIsRepositoryModalOpen(true)
  }

  const handleEditRepository = (repository: Repository) => {
    setEditingRepository(repository)
    setIsRepositoryModalOpen(true)
  }

  const handleDeleteRepository = async (repositoryId: string) => {
    if (!canEditRepositories) {
      message.error('You do not have permission to modify repository settings')
      return
    }

    // Don't allow deleting built-in repositories
    const repo = repositories.find(r => r.id === repositoryId)
    if (repo?.built_in) {
      message.warning('Built-in repositories cannot be deleted')
      return
    }

    try {
      await deleteRepository(repositoryId)
      message.success('Repository removed successfully')
    } catch (error: any) {
      console.error('Failed to delete repository:', error)
      message.error(error?.message || 'Failed to delete repository')
    }
  }

  const handleRepositorySubmit = async (values: any) => {
    if (!canEditRepositories) {
      message.error('You do not have permission to modify repository settings')
      return
    }

    const repositoryData = {
      name: values.name,
      url: values.url,
      auth_type: values.auth_type,
      auth_config: {
        api_key: values.api_key,
        username: values.username,
        password: values.password,
        token: values.token,
      },
      enabled: values.enabled ?? true,
    }

    try {
      if (editingRepository) {
        // Update existing repository
        await updateRepository(editingRepository.id, repositoryData)
        message.success('Repository updated successfully')
      } else {
        // Add new repository
        await createRepository(repositoryData)
        message.success('Repository added successfully')
      }

      setIsRepositoryModalOpen(false)
      repositoryForm.resetFields()
    } catch (error: any) {
      console.error('Failed to save repository:', error)
      message.error(error?.message || 'Failed to save repository')
    }
  }

  const handleRepositoryCancel = () => {
    setIsRepositoryModalOpen(false)
    setEditingRepository(null)
    repositoryForm.resetFields()
  }

  const handleToggleRepository = async (
    repositoryId: string,
    enabled: boolean,
  ) => {
    if (!canEditRepositories) {
      message.error('You do not have permission to modify repository settings')
      return
    }

    try {
      await updateRepository(repositoryId, { enabled })
    } catch (error: any) {
      console.error('Failed to toggle repository:', error)
      message.error(error?.message || 'Failed to toggle repository')
    }
  }

  return (
    <Space direction="vertical" size="large" style={{ width: '100%' }}>
      <div className="mb-6">
        <Title level={2}>{t('settings.modelRepository.title')}</Title>
        <Text type="secondary">
          {t('settings.modelRepository.description')}
        </Text>
      </div>

      {/* Model Repositories */}
      <Card
        title={
          <Flex align="center" gap="middle">
            <CloudDownloadOutlined />
            <span>Model Repositories</span>
          </Flex>
        }
        extra={
          canEditRepositories && (
            <Button
              type="primary"
              icon={<PlusOutlined />}
              onClick={handleAddRepository}
            >
              Add Repository
            </Button>
          )
        }
      >
        <Flex className="flex-col gap-4">
          <div>
            <Text type="secondary" className="block mb-3">
              Configure model repositories with authentication support
            </Text>

            {repositories.length === 0 ? (
              <div className="text-center py-8">
                <CloudDownloadOutlined className="text-4xl mb-2 opacity-50" />
                <div>
                  <Text type="secondary">No repositories configured</Text>
                </div>
                <div>
                  <Text type="secondary">Add a repository to get started</Text>
                </div>
              </div>
            ) : (
              <List
                dataSource={repositories}
                renderItem={(repository: Repository) => (
                  <List.Item
                    actions={
                      canEditRepositories
                        ? [
                            <Switch
                              key="toggle"
                              checked={repository.enabled}
                              onChange={checked =>
                                handleToggleRepository(repository.id, checked)
                              }
                            />,
                            <Button
                              key="test"
                              type="text"
                              icon={<CloudDownloadOutlined />}
                              loading={testing}
                              onClick={() =>
                                testRepositoryConnection(repository)
                              }
                            >
                              Test
                            </Button>,
                            <Button
                              key="edit"
                              type="text"
                              icon={<EditOutlined />}
                              onClick={() => handleEditRepository(repository)}
                            >
                              Edit
                            </Button>,
                            ...(repository.built_in
                              ? []
                              : [
                                  <Button
                                    key="delete"
                                    type="text"
                                    danger
                                    icon={<DeleteOutlined />}
                                    onClick={() =>
                                      handleDeleteRepository(repository.id)
                                    }
                                  >
                                    Delete
                                  </Button>,
                                ]),
                          ]
                        : [
                            <Switch
                              key="toggle"
                              checked={repository.enabled}
                              disabled
                            />,
                          ]
                    }
                  >
                    <List.Item.Meta
                      title={
                        <Flex align="center" gap="small">
                          <Text strong>{repository.name}</Text>
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
                      }
                      description={
                        <Flex className="flex-col gap-1">
                          <Text type="secondary">{repository.url}</Text>
                          <Text type="secondary" className="text-xs">
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
                        </Flex>
                      }
                    />
                  </List.Item>
                )}
              />
            )}
          </div>
        </Flex>
      </Card>

      {/* Repository Add/Edit Modal */}
      <Modal
        title={editingRepository ? 'Edit Repository' : 'Add Repository'}
        open={isRepositoryModalOpen}
        onCancel={handleRepositoryCancel}
        footer={null}
        width={600}
        maskClosable={false}
      >
        <Form
          form={repositoryForm}
          layout="vertical"
          onFinish={handleRepositorySubmit}
        >
          <Form.Item
            name="name"
            label="Repository Name"
            rules={[
              { required: true, message: 'Please enter a repository name' },
            ]}
          >
            <Input placeholder="My Custom Repository" />
          </Form.Item>

          <Form.Item
            name="url"
            label="Repository URL"
            rules={[
              { required: true, message: 'Please enter a repository URL' },
              { type: 'url', message: 'Please enter a valid URL' },
            ]}
          >
            <Input placeholder="https://your-custom-repo.com/models" />
          </Form.Item>

          <Form.Item
            name="auth_type"
            label="Authentication Type"
            rules={[{ required: true }]}
          >
            <Select>
              <Select.Option value="none">No Authentication</Select.Option>
              <Select.Option value="api_key">API Key</Select.Option>
              <Select.Option value="basic_auth">
                Basic Authentication
              </Select.Option>
              <Select.Option value="bearer_token">Bearer Token</Select.Option>
            </Select>
          </Form.Item>

          <Form.Item dependencies={['auth_type']} noStyle>
            {({ getFieldValue }) => {
              const authType = getFieldValue('auth_type')

              if (authType === 'api_key') {
                return (
                  <Form.Item
                    name="api_key"
                    label="API Key"
                    rules={[
                      { required: true, message: 'Please enter your API key' },
                    ]}
                  >
                    <Input.Password
                      placeholder="Enter your API key"
                      iconRender={visible =>
                        visible ? <EyeTwoTone /> : <EyeInvisibleOutlined />
                      }
                    />
                  </Form.Item>
                )
              }

              if (authType === 'basic_auth') {
                return (
                  <>
                    <Form.Item
                      name="username"
                      label="Username"
                      rules={[
                        {
                          required: true,
                          message: 'Please enter your username',
                        },
                      ]}
                    >
                      <Input placeholder="Enter your username" />
                    </Form.Item>
                    <Form.Item
                      name="password"
                      label="Password"
                      rules={[
                        {
                          required: true,
                          message: 'Please enter your password',
                        },
                      ]}
                    >
                      <Input.Password
                        placeholder="Enter your password"
                        iconRender={visible =>
                          visible ? <EyeTwoTone /> : <EyeInvisibleOutlined />
                        }
                      />
                    </Form.Item>
                  </>
                )
              }

              if (authType === 'bearer_token') {
                return (
                  <Form.Item
                    name="token"
                    label="Bearer Token"
                    rules={[
                      {
                        required: true,
                        message: 'Please enter your bearer token',
                      },
                    ]}
                  >
                    <Input.Password
                      placeholder="Enter your bearer token"
                      iconRender={visible =>
                        visible ? <EyeTwoTone /> : <EyeInvisibleOutlined />
                      }
                    />
                  </Form.Item>
                )
              }

              return null
            }}
          </Form.Item>

          {/* Test Connection Section */}
          <Form.Item
            dependencies={[
              'url',
              'auth_type',
              'api_key',
              'username',
              'password',
              'token',
            ]}
            noStyle
          >
            {({ getFieldValue }) => {
              const authType = getFieldValue('auth_type')
              const url = getFieldValue('url')

              // Only show test button if URL is provided and auth is configured (if needed)
              const showTestButton =
                url &&
                (authType === 'none' ||
                  (authType === 'api_key' && getFieldValue('api_key')) ||
                  (authType === 'basic_auth' &&
                    getFieldValue('username') &&
                    getFieldValue('password')) ||
                  (authType === 'bearer_token' && getFieldValue('token')))

              if (showTestButton) {
                return (
                  <Form.Item label="Connection Test">
                    <div>
                      <Text type="secondary" className="block mb-3">
                        Test your repository configuration to ensure it's
                        accessible
                      </Text>
                      <Button
                        type="default"
                        icon={<CloudDownloadOutlined />}
                        loading={testing}
                        onClick={testRepositoryFromForm}
                      >
                        Test Connection
                      </Button>
                    </div>
                  </Form.Item>
                )
              }

              return null
            }}
          </Form.Item>

          <Form.Item
            name="enabled"
            label="Enable Repository"
            valuePropName="checked"
          >
            <Switch />
          </Form.Item>

          <Form.Item>
            <Space>
              <Button
                type="primary"
                htmlType="submit"
                loading={creating || updating}
              >
                {editingRepository ? 'Update' : 'Add'} Repository
              </Button>
              <Button onClick={handleRepositoryCancel}>Cancel</Button>
            </Space>
          </Form.Item>
        </Form>
      </Modal>
    </Space>
  )
}
