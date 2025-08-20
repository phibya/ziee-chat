import {
  CloudDownloadOutlined,
  EyeInvisibleOutlined,
  EyeTwoTone,
} from '@ant-design/icons'
import { App, Button, Form, Input, Select, Switch, Typography } from 'antd'
import { useEffect } from 'react'
import { Drawer } from '../../../common/Drawer'
import {
  createNewAdminModelRepository,
  Stores,
  testAdminModelRepositoryConnection,
  updateAdminModelRepository,
} from '../../../../store'
import {
  closeRepositoryDrawer,
  setRepositoryDrawerLoading,
  useRepositoryDrawerStore,
} from '../../../../store/ui'
import {
  CreateRepositoryRequest,
  UpdateRepositoryRequest,
} from '../../../../types'

const { Text } = Typography

export function RepositoryDrawer() {
  const { message } = App.useApp()
  const [repositoryForm] = Form.useForm()
  const { open, editingRepository, loading } = useRepositoryDrawerStore()
  const { creating, updating, testing } = Stores.AdminRepositories

  // Update repository form when editing
  useEffect(() => {
    if (editingRepository && open) {
      repositoryForm.setFieldsValue({
        name: editingRepository.name,
        url: editingRepository.url,
        auth_type: editingRepository.auth_type,
        api_key: editingRepository.auth_config?.api_key,
        username: editingRepository.auth_config?.username,
        password: editingRepository.auth_config?.password,
        token: editingRepository.auth_config?.token,
        auth_test_api_endpoint:
          editingRepository.auth_config?.auth_test_api_endpoint,
        enabled: editingRepository.enabled,
      })
    } else if (!editingRepository && open) {
      repositoryForm.setFieldsValue({
        auth_type: 'none',
        enabled: true,
      })
    }
  }, [editingRepository, open, repositoryForm])

  const testRepositoryFromForm = async () => {
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
      const testData = {
        name: values.name,
        url: values.url,
        auth_type: values.auth_type,
        auth_config: {
          api_key: values.api_key,
          username: values.username,
          password: values.password,
          token: values.token,
          auth_test_api_endpoint: values.auth_test_api_endpoint,
        },
      }

      const result = await testAdminModelRepositoryConnection(testData)

      if (result.success) {
        message.success(
          result.message || `Connection to ${values.name} successful!`,
        )
      } else {
        message.error(result.message || `Connection to ${values.name} failed`)
      }
    } catch (error: any) {
      console.error('Repository connection test failed:', error)
      message.error(error?.message || `Connection to ${values.name} failed`)
    }
  }

  const handleRepositorySubmit = async (values: any) => {
    setRepositoryDrawerLoading(true)

    let repositoryData: UpdateRepositoryRequest

    if (editingRepository?.built_in) {
      // For built-in repositories, only allow authentication-related fields
      repositoryData = {
        auth_config: {
          api_key: values.api_key,
          username: values.username,
          password: values.password,
          token: values.token,
          auth_test_api_endpoint: values.auth_test_api_endpoint,
        },
      }
    } else {
      // For custom repositories, allow all fields
      repositoryData = {
        name: values.name,
        url: values.url,
        auth_type: values.auth_type,
        auth_config: {
          api_key: values.api_key,
          username: values.username,
          password: values.password,
          token: values.token,
          auth_test_api_endpoint: values.auth_test_api_endpoint,
        },
        enabled: values.enabled ?? true,
      }
    }

    try {
      if (editingRepository) {
        // Update existing repository
        await updateAdminModelRepository(editingRepository.id, repositoryData)
        message.success('Repository updated successfully')
      } else {
        // Add new repository - need full CreateRepositoryRequest
        const createData: CreateRepositoryRequest = {
          name: values.name,
          url: values.url,
          auth_type: values.auth_type,
          auth_config: {
            api_key: values.api_key,
            username: values.username,
            password: values.password,
            token: values.token,
            auth_test_api_endpoint: values.auth_test_api_endpoint,
          },
          enabled: values.enabled ?? true,
        }
        await createNewAdminModelRepository(createData)
        message.success('Repository added successfully')
      }

      closeRepositoryDrawer()
      repositoryForm.resetFields()
    } catch (error: any) {
      console.error('Failed to save repository:', error)
      message.error(error?.message || 'Failed to save repository')
    } finally {
      setRepositoryDrawerLoading(false)
    }
  }

  const handleRepositoryCancel = () => {
    closeRepositoryDrawer()
    repositoryForm.resetFields()
  }
  return (
    <Drawer
      title={
        editingRepository
          ? editingRepository.built_in
            ? 'Edit Built-in Repository (Authentication Only)'
            : 'Edit Repository'
          : 'Add Repository'
      }
      open={open}
      onClose={handleRepositoryCancel}
      footer={[
        <Button key="cancel" onClick={handleRepositoryCancel}>
          Cancel
        </Button>,
        <Button
          key="submit"
          type="primary"
          loading={loading || creating || updating}
          onClick={() => repositoryForm.submit()}
        >
          {editingRepository ? 'Update' : 'Add'} Repository
        </Button>,
      ]}
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
          <Input
            placeholder="My Custom Repository"
            disabled={editingRepository?.built_in}
          />
        </Form.Item>

        <Form.Item
          name="url"
          label="Repository URL"
          rules={[
            { required: true, message: 'Please enter a repository URL' },
            { type: 'url', message: 'Please enter a valid URL' },
          ]}
        >
          <Input
            placeholder="https://your-custom-repo.com/models"
            disabled={editingRepository?.built_in}
          />
        </Form.Item>

        <Form.Item
          name="auth_type"
          label="Authentication Type"
          rules={[{ required: true }]}
        >
          <Select disabled={editingRepository?.built_in}>
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
                <Form.Item name="api_key" label="API Key">
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
                  <Form.Item name="username" label="Username">
                    <Input placeholder="Enter your username" />
                  </Form.Item>
                  <Form.Item name="password" label="Password">
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
                <Form.Item name="token" label="Bearer Token">
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

        <Form.Item
          name="auth_test_api_endpoint"
          label="Authentication Test Endpoint"
          tooltip="Custom endpoint to test authentication. If not provided, the main repository URL will be used for testing."
        >
          <Input
            disabled={editingRepository?.built_in}
            placeholder="https://api.example.com/auth/test"
          />
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
            'auth_test_api_endpoint',
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
          <Switch disabled={editingRepository?.built_in} />
        </Form.Item>
      </Form>
    </Drawer>
  )
}
