import React, { useEffect } from 'react'
import { Layout, Spin } from 'antd'
import { useAuthStore } from '../../store/auth'
import { initializeUserSettings } from '../../store/userSettings'
import { AuthPage } from './AuthPage'

const { Content } = Layout

interface AuthGuardProps {
  children: React.ReactNode
}

export const AuthGuard: React.FC<AuthGuardProps> = ({ children }) => {
  const { isAuthenticated, isLoading, checkInitStatus, getCurrentUser, token } =
    useAuthStore()

  useEffect(() => {
    // Check initialization status on mount
    checkInitStatus()
  }, [])

  useEffect(() => {
    // If we have a token, fetch the current user
    if (token) {
      console.log('Fetching current user with token:', token)
      getCurrentUser()
    }
  }, [token])

  useEffect(() => {
    // Initialize user settings after authentication
    if (isAuthenticated && !isLoading) {
      initializeUserSettings().catch(error => {
        console.error('Failed to initialize user settings:', error)
      })
    }
  }, [isAuthenticated, isLoading])

  // Show loading spinner while checking auth status
  if (isLoading) {
    return (
      <Layout className="min-h-screen">
        <Content className="flex items-center justify-center">
          <Spin size="large" />
        </Content>
      </Layout>
    )
  }

  // Show authentication page if not authenticated
  if (!isAuthenticated) {
    return <AuthPage />
  }

  // Show the protected content
  return <>{children}</>
}
