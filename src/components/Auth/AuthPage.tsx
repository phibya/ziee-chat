import React, { useEffect, useState } from 'react'
import { Layout, Spin, Typography } from 'antd'
import { useAuthStore } from '../../store/auth'
import { LoginForm } from './LoginForm'
import { RegisterForm } from './RegisterForm'

const { Content } = Layout
const { Title } = Typography

type AuthMode = 'login' | 'register' | 'setup'

export const AuthPage: React.FC = () => {
  const [mode, setMode] = useState<AuthMode>('login')
  const { isLoading, needsSetup, isDesktop, isAuthenticated } = useAuthStore()

  useEffect(() => {
    if (needsSetup) {
      setMode('setup')
    } else if (isDesktop) {
      setMode('login')
    } else {
      setMode('login')
    }
  }, [needsSetup, isDesktop])

  // Don't render anything if already authenticated
  if (isAuthenticated) {
    return null
  }

  if (isLoading) {
    return (
      <Layout className="min-h-screen">
        <Content className="flex items-center justify-center">
          <Spin size="large" />
        </Content>
      </Layout>
    )
  }

  return (
    <Layout className="min-h-screen bg-gray-50">
      <Content className="flex items-center justify-center p-4">
        <div className="w-full max-w-md">
          <div className="text-center mb-8">
            <Title level={2}>Welcome</Title>
          </div>

          {mode === 'setup' && <RegisterForm isSetup={true} />}

          {mode === 'login' && (
            <LoginForm onSwitchToRegister={() => setMode('register')} />
          )}

          {mode === 'register' && (
            <RegisterForm onSwitchToLogin={() => setMode('login')} />
          )}
        </div>
      </Content>
    </Layout>
  )
}
