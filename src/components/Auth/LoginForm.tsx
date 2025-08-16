import React, { useEffect } from 'react'
import { Alert, Button, Card, Form, Input, Typography } from 'antd'
import { LockOutlined, UserOutlined } from '@ant-design/icons'
import { useTranslation } from 'react-i18next'
import { Stores, authenticateUser, clearAuthenticationError } from '../../store'
import type { LoginRequest } from '../../types'

const { Text } = Typography

interface LoginFormProps {
  onSwitchToRegister?: () => void
}

export const LoginForm: React.FC<LoginFormProps> = ({ onSwitchToRegister }) => {
  const { t } = useTranslation()
  const [form] = Form.useForm()
  const { isLoading, error, isDesktop } = Stores.Auth

  const onFinish = async (values: LoginRequest) => {
    try {
      clearAuthenticationError()
      await authenticateUser(values)
    } catch (error) {
      // Error is handled by the store
      console.error('Login failed:', error)
    }
  }

  useEffect(() => {
    if (isDesktop) {
      form.setFieldsValue({
        username_or_email: 'root',
        password: '',
      })
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
