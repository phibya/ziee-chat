import React from 'react'
import { Alert, Button, Card, Form, Input, Typography } from 'antd'
import { LockOutlined, UserOutlined } from '@ant-design/icons'
import { useAuthStore } from '../../store/auth'
import type { LoginRequest } from '../../types'

const { Title, Text } = Typography

interface LoginFormProps {
  onSwitchToRegister?: () => void
}

export const LoginForm: React.FC<LoginFormProps> = ({ onSwitchToRegister }) => {
  const [form] = Form.useForm()
  const { login, isLoading, error, clearError, isDesktop } = useAuthStore()

  const onFinish = async (values: LoginRequest) => {
    try {
      clearError()
      await login(values)
    } catch (error) {
      // Error is handled by the store
      console.error('Login failed:', error)
    }
  }

  return (
    <Card className="w-full max-w-md mx-auto">
      <div className="text-center mb-6">
        <Title level={3}>Sign In</Title>
        {isDesktop && (
          <Text type="secondary">Desktop App - Auto Authentication</Text>
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
        name="login"
        onFinish={onFinish}
        layout="vertical"
        size="large"
        autoComplete="off"
      >
        <Form.Item
          label="Username or Email"
          name="username_or_email"
          rules={[
            { required: true, message: 'Please input your username or email!' },
          ]}
        >
          <Input
            prefix={<UserOutlined />}
            placeholder="Enter username or email"
            autoComplete="username"
          />
        </Form.Item>

        <Form.Item
          label="Password"
          name="password"
          rules={[{ required: true, message: 'Please input your password!' }]}
        >
          <Input.Password
            prefix={<LockOutlined />}
            placeholder="Enter password"
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
            Sign In
          </Button>
        </Form.Item>

        {!isDesktop && onSwitchToRegister && (
          <div className="text-center">
            <Text type="secondary">
              Don't have an account?{' '}
              <Button type="link" onClick={onSwitchToRegister} className="p-0">
                Sign Up
              </Button>
            </Text>
          </div>
        )}
      </Form>
    </Card>
  )
}
