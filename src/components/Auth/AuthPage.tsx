import React, { useEffect, useState } from 'react'
import { Layout, Spin, Typography } from 'antd'
import { useShallow } from 'zustand/react/shallow'
import { useAuthStore } from '../../store/auth'
import { useAdminStore } from '../../store/admin'
import { LoginForm } from './LoginForm'
import { RegisterForm } from './RegisterForm'

const { Content } = Layout
const { Title } = Typography

type AuthMode = 'login' | 'register' | 'setup'

export const AuthPage: React.FC = () => {
  const [mode, setMode] = useState<AuthMode>('login')
  const { isLoading, needsSetup, isDesktop, isAuthenticated } = useAuthStore()

  // Get registration status from admin store
  const { registrationEnabled, loadUserRegistrationSettings } = useAdminStore(
    useShallow(state => ({
      registrationEnabled: state.userRegistrationEnabled,
      loadUserRegistrationSettings: state.loadUserRegistrationSettings,
    })),
  )

  const [checkingRegistration, setCheckingRegistration] = useState(false)

  // Check registration status for web app
  useEffect(() => {
    const checkRegistrationStatus = async () => {
      if (!needsSetup && !isDesktop) {
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
    }

    checkRegistrationStatus()
  }, [needsSetup, isDesktop, loadUserRegistrationSettings])

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
