import { Card, Divider, Flex, message, Select, Space, Typography } from 'antd'
import { useEffect, useState } from 'react'
import { useAppearanceSettings } from '../../../store'

const { Title, Text } = Typography

export function AppearanceSettings() {
  const [isMobile, setIsMobile] = useState(false)
  const {
    theme,
    componentSize,
    language,
    setTheme,
    setComponentSize,
    setLanguage,
    loading,
  } = useAppearanceSettings()

  useEffect(() => {
    const checkMobile = () => {
      setIsMobile(window.innerWidth < 768)
    }

    checkMobile()
    window.addEventListener('resize', checkMobile)

    return () => window.removeEventListener('resize', checkMobile)
  }, [])

  const handleThemeChange = async (newTheme: 'light' | 'dark' | 'system') => {
    try {
      await setTheme(newTheme)
      message.success('Theme updated successfully')
    } catch (error) {
      message.error('Failed to update theme')
    }
  }

  const handleComponentSizeChange = async (
    newComponentSize: 'small' | 'medium' | 'large',
  ) => {
    try {
      await setComponentSize(newComponentSize)
      message.success('Component size updated successfully')
    } catch (error) {
      message.error('Failed to update component size')
    }
  }

  const handleLanguageChange = async (newLanguage: 'en' | 'vi') => {
    try {
      await setLanguage(newLanguage)
      message.success('Language updated successfully')
    } catch (error) {
      message.error('Failed to update language')
    }
  }

  return (
    <Space direction="vertical" size="large" style={{ width: '100%' }}>
      <Title level={3}>Appearance</Title>

      <Card title="Theme & Display">
        <Space direction="vertical" size="middle" style={{ width: '100%' }}>
          <Flex
            justify="space-between"
            align={isMobile ? 'flex-start' : 'center'}
            vertical={isMobile}
            gap={isMobile ? 'small' : 0}
          >
            <div>
              <Text strong>Theme</Text>
              <div>
                <Text type="secondary">
                  Choose your preferred theme or match the OS theme.
                </Text>
              </div>
            </div>
            <Select
              value={theme}
              onChange={handleThemeChange}
              loading={loading}
              style={{ minWidth: 120 }}
              options={[
                { value: 'light', label: 'Light' },
                { value: 'dark', label: 'Dark' },
                { value: 'system', label: 'System' },
              ]}
            />
          </Flex>
          <Divider style={{ margin: 0 }} />
          <Flex
            justify="space-between"
            align={isMobile ? 'flex-start' : 'center'}
            vertical={isMobile}
            gap={isMobile ? 'small' : 0}
          >
            <div>
              <Text strong>Component Size</Text>
              <div>
                <Text type="secondary">
                  Adjust the size of UI components throughout the app.
                </Text>
              </div>
            </div>
            <Select
              value={componentSize}
              onChange={handleComponentSizeChange}
              loading={loading}
              style={{ minWidth: 120 }}
              options={[
                { value: 'small', label: 'Small' },
                { value: 'medium', label: 'Medium' },
                { value: 'large', label: 'Large' },
              ]}
            />
          </Flex>
          <Divider style={{ margin: 0 }} />
          <Flex
            justify="space-between"
            align={isMobile ? 'flex-start' : 'center'}
            vertical={isMobile}
            gap={isMobile ? 'small' : 0}
          >
            <div>
              <Text strong>Language</Text>
              <div>
                <Text type="secondary">
                  Choose your preferred language for the interface.
                </Text>
              </div>
            </div>
            <Select
              value={language}
              onChange={handleLanguageChange}
              loading={loading}
              style={{ minWidth: 120 }}
              options={[
                { value: 'en', label: 'English' },
                { value: 'vi', label: 'Tiếng Việt' },
              ]}
            />
          </Flex>
        </Space>
      </Card>
    </Space>
  )
}
