import React, { useEffect, useState } from 'react'
import { useNavigate, useSearchParams } from 'react-router-dom'
import { Alert, Card, Result, Spin } from 'antd'
import { handleOAuthCallback } from '../../store'

export const OAuthCallback: React.FC = () => {
  const [searchParams] = useSearchParams()
  const navigate = useNavigate()
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    const processCallback = async () => {
      try {
        // Extract OAuth callback parameters from URL
        const code = searchParams.get('code')
        const state = searchParams.get('state')

        // Get session data from sessionStorage (stored during initiation)
        const sessionKey = sessionStorage.getItem('oauth_session_key')
        const providerId = sessionStorage.getItem('oauth_provider_id')

        // Validate required parameters
        if (!code || !state || !sessionKey || !providerId) {
          setError(
            'Invalid OAuth callback: missing required parameters. Please try logging in again.',
          )
          setLoading(false)
          return
        }

        // Call the OAuth callback handler
        await handleOAuthCallback(providerId, code, state, sessionKey)

        // Successful authentication - redirect to app
        navigate('/')
      } catch (err) {
        console.error('OAuth callback error:', err)
        setError(
          err instanceof Error
            ? err.message
            : 'Authentication failed. Please try again.',
        )
        setLoading(false)
      }
    }

    processCallback()
  }, [searchParams, navigate])

  if (loading) {
    return (
      <div className="w-full h-screen flex items-center justify-center">
        <Card className="w-full max-w-md mx-auto text-center">
          <Spin size="large" />
          <div className="mt-4">
            <p>Completing authentication...</p>
            <p className="text-gray-500 text-sm mt-2">
              Please wait while we verify your credentials
            </p>
          </div>
        </Card>
      </div>
    )
  }

  if (error) {
    return (
      <div className="w-full h-screen flex items-center justify-center">
        <Card className="w-full max-w-md mx-auto">
          <Result
            status="error"
            title="Authentication Failed"
            subTitle={error}
            extra={
              <div className="space-y-2">
                <button
                  onClick={() => navigate('/login')}
                  className="w-full px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
                >
                  Back to Login
                </button>
              </div>
            }
          />
          <div className="mt-4">
            <Alert
              message="Troubleshooting Tips"
              description={
                <ul className="list-disc list-inside text-sm">
                  <li>Make sure you allowed the requested permissions</li>
                  <li>Try clearing your browser cookies and cache</li>
                  <li>Contact your administrator if the problem persists</li>
                </ul>
              }
              type="info"
              showIcon
            />
          </div>
        </Card>
      </div>
    )
  }

  return null
}
