import React, { useEffect, useState } from 'react'
import { Alert, Button, Card, Form, Input, Typography } from 'antd'
import { useTranslation } from 'react-i18next'
import { LockOutlined, MailOutlined, UserOutlined } from '@ant-design/icons'
import { useShallow } from 'zustand/react/shallow'
import { useAuthStore } from '../../store/auth'
import { useAdminStore } from '../../store/admin'
import type { CreateUserRequest } from '../../types'

const { Title, Text } = Typography

interface RegisterFormProps {
  onSwitchToLogin?: () => void
  isSetup?: boolean
}

export const RegisterForm: React.FC<RegisterFormProps> = ({
  onSwitchToLogin,
  isSetup = false,
}) => {
  const { t } = useTranslation()
  const [form] = Form.useForm()
  const [checkingRegistration, setCheckingRegistration] = useState(false)
  const { register, setupApp, isLoading, error, clearError, isDesktop } =
    useAuthStore()

  // Get registration status from admin store
  const { registrationEnabled, loadUserRegistrationSettings } = useAdminStore(
    useShallow(state => ({
      registrationEnabled: state.userRegistrationEnabled,
      loadUserRegistrationSettings: state.loadUserRegistrationSettings,
    })),
  )

  // Check registration status for web app (except for setup mode)
  useEffect(() => {
    if (!isSetup && !isDesktop) {
      const checkRegistrationStatus = async () => {
        setCheckingRegistration(true)
        try {
          await loadUserRegistrationSettings()
        } catch {
          // If we can't check status, registration status will remain default
          console.warn('Failed to load registration status')
        } finally {
          setCheckingRegistration(false)
        }
      }

      checkRegistrationStatus()
    }
  }, [isSetup, isDesktop, loadUserRegistrationSettings])

  const onFinish = async (values: CreateUserRequest) => {
    try {
      clearError()
      if (isSetup) {
        await setupApp(values)
      } else {
        await register(values)
      }
    } catch (error) {
      // Error is handled by the store
      console.error('Registration failed:', error)
    }
  }

  const title = isSetup ? 'Setup Admin Account' : 'Create Account'
  const submitText = isSetup ? 'Setup App' : 'Sign Up'

  // Show loading state while checking registration status
  if (checkingRegistration) {
    return (
      <Card className="w-full max-w-md mx-auto">
        <div className="text-center p-4">
          <Text type="secondary">Checking registration status...</Text>
        </div>
      </Card>
    )
  }

  // Show disabled message if registration is not enabled
  if (!isSetup && !isDesktop && !registrationEnabled) {
    return (
      <Card className="w-full max-w-md mx-auto">
        <div className="text-center">
          <Title level={3}>Registration Disabled</Title>
          <Text type="secondary">
            User registration is currently disabled by the administrator.
          </Text>
          {onSwitchToLogin && (
            <div className="mt-4">
              <Button type="primary" onClick={onSwitchToLogin}>
                Back to Login
              </Button>
            </div>
          )}
        </div>
      </Card>
    )
  }

  return (
    <Card className="w-full max-w-md mx-auto">
      <div className="text-center mb-6">
        <Title level={3}>{title}</Title>
        {isSetup && (
          <Text type="secondary">
            Create the first admin account to get started
          </Text>
        )}
      </div>

      {error && (
        <Alert
          message={error}
          type="error"
          showIcon
          closable
          onClose={clearError}
          className="mb-4"
        />
      )}

      <Form
        form={form}
        name="register"
        onFinish={onFinish}
        layout="vertical"
        size="large"
        autoComplete="off"
      >
        <Form.Item
          label={t('auth.username')}
          name="username"
          rules={[
            { required: true, message: 'Please input your username!' },
            { min: 3, message: 'Username must be at least 3 characters long!' },
          ]}
        >
          <Input
            prefix={<UserOutlined />}
            placeholder={t('auth.enterUsername')}
            autoComplete="username"
          />
        </Form.Item>

        <Form.Item
          label={t('auth.email')}
          name="email"
          rules={[
            { required: true, message: 'Please input your email!' },
            { type: 'email', message: 'Please enter a valid email address!' },
          ]}
        >
          <Input
            prefix={<MailOutlined />}
            placeholder={t('auth.enterEmailAddress')}
            autoComplete="email"
          />
        </Form.Item>

        <Form.Item
          label={t('auth.password')}
          name="password"
          rules={[
            { required: true, message: 'Please input your password!' },
            { min: 6, message: 'Password must be at least 6 characters long!' },
          ]}
        >
          <Input.Password
            prefix={<LockOutlined />}
            placeholder={t('auth.enterPassword')}
            autoComplete="new-password"
          />
        </Form.Item>

        <Form.Item
          label={t('auth.confirmPassword')}
          name="confirmPassword"
          dependencies={['password']}
          rules={[
            { required: true, message: 'Please confirm your password!' },
            ({ getFieldValue }) => ({
              validator(_, value) {
                if (!value || getFieldValue('password') === value) {
                  return Promise.resolve()
                }
                return Promise.reject(new Error('Passwords do not match!'))
              },
            }),
          ]}
        >
          <Input.Password
            prefix={<LockOutlined />}
            placeholder={t('auth.confirmPasswordPlaceholder')}
            autoComplete="new-password"
          />
        </Form.Item>

        <Form.Item>
          <Button
            type="primary"
            htmlType="submit"
            loading={isLoading}
            className="w-full"
          >
            {submitText}
          </Button>
        </Form.Item>

        {!isSetup && !isDesktop && onSwitchToLogin && (
          <div className="text-center">
            <Text type="secondary">
              Already have an account?{' '}
              <Button type="link" onClick={onSwitchToLogin} className="p-0">
                Sign In
              </Button>
            </Text>
          </div>
        )}
      </Form>
    </Card>
  )
}
