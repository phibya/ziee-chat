import { Button, Card, Divider, Flex, Space, Typography, Select, InputNumber, message } from 'antd'
import { useEffect, useState } from 'react'
import { useAppearanceSettings } from '../../../store/userSettings'

const { Title, Text } = Typography

export function AppearanceSettings() {
  const [isMobile, setIsMobile] = useState(false)
  const {
    theme,
    fontSize,
    setTheme,
    setFontSize,
    loading
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

  const handleFontSizeChange = async (newFontSize: number | null) => {
    if (newFontSize === null) return
    
    try {
      await setFontSize(newFontSize)
      message.success('Font size updated successfully')
    } catch (error) {
      message.error('Failed to update font size')
    }
  }

  const getThemeLabel = (theme: 'light' | 'dark' | 'system') => {
    switch (theme) {
      case 'light': return 'Light'
      case 'dark': return 'Dark'
      case 'system': return 'System'
      default: return 'System'
    }
  }

  const getFontSizeLabel = (fontSize: number) => {
    if (fontSize <= 12) return 'Small'
    if (fontSize <= 16) return 'Medium'
    return 'Large'
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
                <Text type="secondary">Choose your preferred theme or match the OS theme.</Text>
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
                { value: 'system', label: 'System' }
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
              <Text strong>Font Size</Text>
              <div>
                <Text type="secondary">Adjust the app's font size (8-32px).</Text>
              </div>
            </div>
            <Space.Compact>
              <InputNumber
                value={fontSize}
                onChange={handleFontSizeChange}
                min={8}
                max={32}
                step={1}
                style={{ width: 80 }}
                loading={loading}
              />
              <Button 
                type="default" 
                disabled
                style={{ minWidth: 80 }}
              >
                {getFontSizeLabel(fontSize)}
              </Button>
            </Space.Compact>
          </Flex>
        </Space>
      </Card>
    </Space>
  )
}
