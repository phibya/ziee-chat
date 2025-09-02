import React, { useEffect, useState } from 'react'
import { Layout, Typography } from 'antd'
import { Stores } from '../../store'
import { LoginForm } from './LoginForm'
import { RegisterForm } from './RegisterForm'

const { Content } = Layout
const { Title } = Typography

type AuthMode = 'login' | 'register' | 'setup'

export const AuthPage: React.FC = () => {
  const [mode, setMode] = useState<AuthMode>('login')
  const { needsSetup, isDesktop, isAuthenticated, allowRegistration } =
    Stores.Auth

  useEffect(() => {
    if (needsSetup) {
      setMode('setup')
    } else if (isDesktop) {
      setMode('login')
    } else {
      setMode('login')
    }
  }, [needsSetup, isDesktop])

  const handleSwitchToRegister = () => {
    if (!allowRegistration) {
      return // Don't allow switching if registration is disabled
    }
    setMode('register')
  }

  // Don't render anything if already authenticated
  if (isAuthenticated) {
    return null
  }

  return (
    <Layout className="min-h-screen">
      <Content className="flex items-center justify-center p-4">
        <div className="w-full max-w-md">
          <div className="text-center mb-8">
            <Title level={2}>Welcome</Title>
          </div>

          {mode === 'setup' && <RegisterForm isSetup={true} />}

          {mode === 'login' && (
            <LoginForm
              onSwitchToRegister={
                allowRegistration ? handleSwitchToRegister : undefined
              }
            />
          )}

          {mode === 'register' && allowRegistration && (
            <RegisterForm onSwitchToLogin={() => setMode('login')} />
          )}
        </div>
      </Content>
    </Layout>
  )
}
