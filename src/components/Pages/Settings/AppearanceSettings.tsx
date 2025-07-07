import {
  Button,
  Card,
  Col,
  Divider,
  Flex,
  Row,
  Space,
  Switch,
  Typography,
} from 'antd'
import { useEffect, useState } from 'react'
import { CopyOutlined } from '@ant-design/icons'

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
  const ColorPicker = ({ colors }: { colors: string[] }) => (
    <Space>
      {colors.map((color, index) => (
        <Button
          key={index}
          shape="circle"
          size="small"
          style={{ backgroundColor: color, borderColor: color }}
        />
      ))}
      <Button type="text" size="small" shape="circle">
        ✎
      </Button>
    </Space>
  )

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

      <Card title="Colors">
        <Space direction="vertical" size="middle" style={{ width: '100%' }}>
          <Flex
            justify="space-between"
            align={isMobile ? 'flex-start' : 'center'}
            vertical={isMobile}
            gap={isMobile ? 'small' : 0}
          >
            <div>
              <Text strong>Window Background</Text>
              <div>
                <Text type="secondary">
                  Set the app window's background color.
                </Text>
              </div>
            </div>
            <ColorPicker
              colors={[
                '#f5222d',
                '#1890ff',
                '#722ed1',
                '#fadb14',
                '#52c41a',
                '#fa8c16',
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
              <Text strong>App Main View</Text>
              <div>
                <Text type="secondary">
                  Set the main content area's background color.
                </Text>
              </div>
            </div>
            <ColorPicker colors={['#ffffff', '#262626']} />
          </Flex>
          <Divider style={{ margin: 0 }} />
          <Flex
            justify="space-between"
            align={isMobile ? 'flex-start' : 'center'}
            vertical={isMobile}
            gap={isMobile ? 'small' : 0}
          >
            <div>
              <Text strong>Primary</Text>
              <div>
                <Text type="secondary">
                  Set the primary color for UI components.
                </Text>
              </div>
            </div>
            <ColorPicker
              colors={['#fa8c16', '#1890ff', '#52c41a', '#722ed1']}
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
              <Text strong>Accent</Text>
              <div>
                <Text type="secondary">
                  Set the accent color for UI highlights.
                </Text>
              </div>
            </div>
            <ColorPicker colors={['#1890ff', '#f5222d', '#52c41a']} />
          </Flex>
          <Divider style={{ margin: 0 }} />
          <Flex
            justify="space-between"
            align={isMobile ? 'flex-start' : 'center'}
            vertical={isMobile}
            gap={isMobile ? 'small' : 0}
          >
            <div>
              <Text strong>Destructive</Text>
              <div>
                <Text type="secondary">
                  Set the color for destructive actions.
                </Text>
              </div>
            </div>
            <ColorPicker
              colors={['#f5222d', '#cf1322', '#722ed1', '#eb2f96']}
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
              <Text strong>Reset to Default</Text>
              <div>
                <Text type="secondary">
                  Reset all appearance settings to default.
                </Text>
              </div>
            </div>
            <Button type="primary" danger>
              Reset
            </Button>
          </Flex>
        </Space>
      </Card>

      <Card title="Chat Layout">
        <div style={{ marginBottom: 16 }}>
          <Text strong>Chat Width</Text>
          <div>
            <Text type="secondary">Customize the width of the chat view.</Text>
          </div>
        </div>
        <Row gutter={[16, 16]}>
          <Col xs={24} md={12}>
            <Card
              size="small"
              className="selected-card"
              title={<Text type="secondary">Compact Width</Text>}
            >
              <Space
                direction="vertical"
                size="small"
                style={{ width: '100%' }}
              >
                {[...Array(5)].map((_, i) => (
                  <div key={i} className="chat-line" />
                ))}
                <div className="chat-input">
                  <Text type="secondary" style={{ fontSize: 10 }}>
                    Ask me anything...
                  </Text>
                </div>
              </Space>
            </Card>
          </Col>
          <Col xs={24} md={12}>
            <Card size="small" title={<Text type="secondary">Full Width</Text>}>
              <Space
                direction="vertical"
                size="small"
                style={{ width: '100%' }}
              >
                {[...Array(5)].map((_, i) => (
                  <div key={i} className="chat-line" />
                ))}
                <div className="chat-input">
                  <Text type="secondary" style={{ fontSize: 10 }}>
                    Ask me anything...
                  </Text>
                </div>
              </Space>
            </Card>
          </Col>
        </Row>
      </Card>

      <Card title="Code Blocks">
        <Space direction="vertical" size="middle" style={{ width: '100%' }}>
          <Flex
            justify="space-between"
            align={isMobile ? 'flex-start' : 'center'}
            vertical={isMobile}
            gap={isMobile ? 'small' : 0}
          >
            <div>
              <Text strong>Code Block Theme</Text>
              <div>
                <Text type="secondary">
                  Choose a syntax highlighting style.
                </Text>
              </div>
            </div>
            <Button type="default">VSCode Dark+</Button>
          </Flex>

          <Card className="code-preview">
            <Flex
              justify="space-between"
              align="center"
              style={{ marginBottom: 8 }}
            >
              <Text type="secondary">Preview</Text>
              <Space>
                <Text type="secondary">Typescript</Text>
                <Button size="small" icon={<CopyOutlined />} type="text">
                  Copy
                </Button>
              </Space>
            </Flex>
            <div>
              <Text code>1 // Example code for preview</Text>
              <br />
              <Text code>2 function greeting(name: string)</Text>
              <br />
              <Text code>3 return `Hello, {'${name}'}!`;</Text>
              <br />
              <Text code>4</Text>
              <br />
              <Text code>5</Text>
              <br />
              <Text code>6 // Call the function</Text>
              <br />
              <Text code>7 const message = greeting('Jan');</Text>
              <br />
              <Text code>8 console.log(message); // Outputs: Hello, Jan!</Text>
            </div>
          </Card>

          <Divider style={{ margin: 0 }} />

          <Flex
            justify="space-between"
            align={isMobile ? 'flex-start' : 'center'}
            vertical={isMobile}
            gap={isMobile ? 'small' : 0}
          >
            <div>
              <Text strong>Show Line Numbers</Text>
              <div>
                <Text type="secondary">
                  Display line numbers in code blocks.
                </Text>
              </div>
            </div>
            <Switch size="small" defaultChecked />
          </Flex>

          <Divider style={{ margin: 0 }} />

          <Flex
            justify="space-between"
            align={isMobile ? 'flex-start' : 'center'}
            vertical={isMobile}
            gap={isMobile ? 'small' : 0}
          >
            <div>
              <Text strong>Reset Code Block Style</Text>
              <div>
                <Text type="secondary">Reset code block style to default.</Text>
              </div>
            </div>
            <Button type="primary" danger>
              Reset
            </Button>
          </Flex>
        </Space>
      </Card>
    </Space>
  )
}
