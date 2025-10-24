import React, { useEffect } from 'react'
import {
  Alert,
  Button,
  Card,
  Divider,
  Flex,
  Form,
  Input,
  Typography,
} from 'antd'
import {
  GoogleOutlined,
  LockOutlined,
  UserOutlined,
  WindowsOutlined,
} from '@ant-design/icons'
import { useTranslation } from 'react-i18next'
import {
  authenticateUser,
  clearAuthenticationError,
  initiateOAuthLogin,
  loadEnabledProviders,
  Stores,
} from '../../store'
import type { LoginRequest } from '../../types'

const { Text } = Typography

interface LoginFormProps {
  onSwitchToRegister?: () => void
}

export const LoginForm: React.FC<LoginFormProps> = ({ onSwitchToRegister }) => {
  const { t } = useTranslation()
  const [form] = Form.useForm()
  const {
    isLoading,
    error,
    isDesktop,
    availableProviders,
    oauthLoading,
    oauthError,
  } = Stores.Auth

  const onFinish = async (values: LoginRequest) => {
    try {
      clearAuthenticationError()
      await authenticateUser(values)
    } catch (error) {
      // Error is handled by the store
      console.error('Login failed:', error)
    }
  }

  const handleOAuthLogin = async (providerId: string) => {
    const redirectUri = `${window.location.origin}/auth/callback`
    try {
      await initiateOAuthLogin(providerId, redirectUri)
    } catch (error) {
      console.error('OAuth initiation failed:', error)
    }
  }

  const getProviderIcon = (providerName: string) => {
    const name = providerName.toLowerCase()
    if (name.includes('google')) return <GoogleOutlined />
    if (name.includes('microsoft') || name.includes('azure'))
      return <WindowsOutlined />
    return null
  }

  useEffect(() => {
    if (isDesktop) {
      form.setFieldsValue({
        username_or_email: 'root',
        password: '',
      })
    }
  }, [isDesktop])

  useEffect(() => {
    // Load enabled OAuth providers on mount
    if (!isDesktop) {
      loadEnabledProviders()
    }
  }, [isDesktop])

  return (
    <Card className="w-full max-w-md mx-auto">
      {error && (
        <div className={'py-4'}>
          <Alert
            message={error}
            type="error"
            showIcon
            closable
            onClose={clearAuthenticationError}
          />
        </div>
      )}

      <Form
        form={form}
        name="login"
        onFinish={onFinish}
        layout="vertical"
        size="large"
        autoComplete="off"
      >
        <Form.Item
          label={t('auth.usernameOrEmail')}
          name="username_or_email"
          rules={[
            { required: true, message: t('auth.usernameOrEmailRequired') },
          ]}
          hidden={isDesktop}
        >
          <Input
            prefix={<UserOutlined />}
            placeholder={t('auth.usernameOrEmailPlaceholder')}
            autoComplete="username"
          />
        </Form.Item>

        <Form.Item
          label={t('auth.password')}
          name="password"
          rules={[{ required: true, message: t('auth.passwordRequired') }]}
        >
          <Input.Password
            prefix={<LockOutlined />}
            placeholder={t('auth.passwordPlaceholder')}
            autoComplete="current-password"
          />
        </Form.Item>

        <Form.Item>
          <Button
            type="primary"
            htmlType="submit"
            loading={isLoading}
            className="w-full"
          >
            {t('auth.signIn')}
          </Button>
        </Form.Item>

        {!isDesktop && availableProviders.length > 0 && (
          <>
            <Divider>Or sign in with</Divider>
            {oauthError && (
              <Alert
                message={oauthError}
                type="error"
                showIcon
                closable
                className="mb-4"
              />
            )}
            <Flex vertical gap="small">
              {availableProviders.map(provider => (
                <Button
                  key={provider.id}
                  size="large"
                  icon={getProviderIcon(provider.name)}
                  onClick={() => handleOAuthLogin(provider.id)}
                  loading={oauthLoading}
                >
                  Continue with {provider.name}
                </Button>
              ))}
            </Flex>
          </>
        )}

        {!isDesktop && onSwitchToRegister && (
          <div className="text-center">
            <Text type="secondary">
              {t('auth.dontHaveAccount')}{' '}
              <Button type="link" onClick={onSwitchToRegister} className="p-0">
                {t('auth.signUp')}
              </Button>
            </Text>
          </div>
        )}
      </Form>
    </Card>
  )
}
