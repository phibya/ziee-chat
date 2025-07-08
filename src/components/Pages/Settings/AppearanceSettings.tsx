import { Button, Card, Divider, Flex, Space, Typography } from 'antd'
import { useEffect, useState } from 'react'

const { Title, Text } = Typography

export function AppearanceSettings() {
  const [isMobile, setIsMobile] = useState(false)

  useEffect(() => {
    const checkMobile = () => {
      setIsMobile(window.innerWidth < 768)
    }

    checkMobile()
    window.addEventListener('resize', checkMobile)

    return () => window.removeEventListener('resize', checkMobile)
  }, [])

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
                <Text type="secondary">Match the OS theme.</Text>
              </div>
            </div>
            <Button type="default">System</Button>
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
                <Text type="secondary">Adjust the app's font size.</Text>
              </div>
            </div>
            <Button type="default">Medium</Button>
          </Flex>
        </Space>
      </Card>
    </Space>
  )
}
