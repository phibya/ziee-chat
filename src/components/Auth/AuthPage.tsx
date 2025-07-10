import React, { useEffect, useState } from 'react'
import { Layout, Spin, Typography } from 'antd'
import { useAuthStore } from '../../store/auth'
import { LoginForm } from './LoginForm'
import { RegisterForm } from './RegisterForm'
import { ApiClient } from '../../api/client'

const { Content } = Layout
const { Title } = Typography

type AuthMode = 'login' | 'register' | 'setup'

export const AuthPage: React.FC = () => {
  const [mode, setMode] = useState<AuthMode>('login')
  const [registrationEnabled, setRegistrationEnabled] = useState(true)
  const [checkingRegistration, setCheckingRegistration] = useState(false)
  const { isLoading, needsSetup, isDesktop, isAuthenticated } = useAuthStore()

  // Check registration status for web app
  useEffect(() => {
    const checkRegistrationStatus = async () => {
      if (!needsSetup && !isDesktop) {
        setCheckingRegistration(true)
        try {
          const response = await ApiClient.Config.getUserRegistrationStatus()
          setRegistrationEnabled(response.enabled)
        } catch {
          // If we can't check status, assume registration is enabled
          setRegistrationEnabled(true)
        } finally {
          setCheckingRegistration(false)
        }
      }
    }

    checkRegistrationStatus()
  }, [needsSetup, isDesktop])

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
    if (!registrationEnabled) {
      return // Don't allow switching if registration is disabled
    }
    setMode('register')
  }

  // Don't render anything if already authenticated
  if (isAuthenticated) {
    return null
  }

  if (isLoading || checkingRegistration) {
    return (
      <Layout className="min-h-screen">
        <Content className="flex items-center justify-center">
          <Spin size="large" />
        </Content>
      </Layout>
    )
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
                registrationEnabled ? handleSwitchToRegister : undefined
              }
            />
          )}

          {mode === 'register' && registrationEnabled && (
            <RegisterForm onSwitchToLogin={() => setMode('login')} />
          )}
        </div>
      </Content>
    </Layout>
  )
}
