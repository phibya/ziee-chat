import {
  Button,
  Switch,
  Typography,
  Divider,
  Space,
  Flex,
  Card,
  Row,
  Col,
} from 'antd'
import { CopyOutlined } from '@ant-design/icons'

const { Title, Text } = Typography

export function AppearanceSettings() {
  const ColorPicker = ({ colors }: { colors: string[] }) => (
    <Space>
      {colors.map((color, index) => (
        <div
          key={index}
          style={{
            width: 24,
            height: 24,
            borderRadius: '50%',
            backgroundColor: color,
            cursor: 'pointer',
            border: '2px solid #d9d9d9',
          }}
        />
      ))}
      <Button
        type="text"
        size="small"
        style={{ width: 24, height: 24, minWidth: 24, padding: 0 }}
      >
        ✎
      </Button>
    </Space>
  )

  return (
    <Space direction="vertical" size="large" style={{ width: '100%' }}>
      <Title level={3}>Appearance</Title>

      <Card title="Theme & Display">
        <Space direction="vertical" size="middle" style={{ width: '100%' }}>
          <Flex justify="space-between" align="center">
            <div>
              <Text strong>Theme</Text>
              <div>
                <Text type="secondary">Match the OS theme.</Text>
              </div>
            </div>
            <Button type="default">System</Button>
          </Flex>
          <Divider style={{ margin: 0 }} />
          <Flex justify="space-between" align="center">
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
          <Flex justify="space-between" align="center">
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
          <Flex justify="space-between" align="center">
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
          <Flex justify="space-between" align="center">
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
          <Flex justify="space-between" align="center">
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
          <Flex justify="space-between" align="center">
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
          <Flex justify="space-between" align="center">
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
        <Row gutter={16}>
          <Col span={12}>
            <Card
              size="small"
              style={{ border: '2px solid #eb2f96' }}
              title={<Text style={{ fontSize: 12 }}>Compact Width</Text>}
            >
              <Space
                direction="vertical"
                size="small"
                style={{ width: '100%' }}
              >
                {[...Array(5)].map((_, i) => (
                  <div
                    key={i}
                    style={{
                      height: 8,
                      backgroundColor: '#f0f0f0',
                      borderRadius: 4,
                    }}
                  />
                ))}
                <div
                  style={{
                    padding: 8,
                    backgroundColor: '#f0f0f0',
                    borderRadius: 4,
                    textAlign: 'center',
                  }}
                >
                  <Text style={{ fontSize: 10 }} type="secondary">
                    Ask me anything...
                  </Text>
                </div>
              </Space>
            </Card>
          </Col>
          <Col span={12}>
            <Card
              size="small"
              title={<Text style={{ fontSize: 12 }}>Full Width</Text>}
            >
              <Space
                direction="vertical"
                size="small"
                style={{ width: '100%' }}
              >
                {[...Array(5)].map((_, i) => (
                  <div
                    key={i}
                    style={{
                      height: 8,
                      backgroundColor: '#f0f0f0',
                      borderRadius: 4,
                    }}
                  />
                ))}
                <div
                  style={{
                    padding: 8,
                    backgroundColor: '#f0f0f0',
                    borderRadius: 4,
                    textAlign: 'center',
                  }}
                >
                  <Text style={{ fontSize: 10 }} type="secondary">
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
          <Flex justify="space-between" align="center">
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

          <Card style={{ backgroundColor: '#1f1f1f' }}>
            <Flex
              justify="space-between"
              align="center"
              style={{ marginBottom: 8 }}
            >
              <Text style={{ color: '#8c8c8c', fontSize: 12 }}>Preview</Text>
              <Space>
                <Text style={{ color: '#8c8c8c', fontSize: 12 }}>
                  Typescript
                </Text>
                <Button
                  size="small"
                  type="text"
                  icon={<CopyOutlined />}
                  style={{ color: '#8c8c8c' }}
                >
                  Copy
                </Button>
              </Space>
            </Flex>
            <div style={{ fontFamily: 'monospace', fontSize: 12 }}>
              <div style={{ color: '#8c8c8c' }}>
                1{' '}
                <Text style={{ color: '#6a9955' }}>
                  // Example code for preview
                </Text>
              </div>
              <div style={{ color: '#8c8c8c' }}>
                2 <Text style={{ color: '#569cd6' }}>function</Text>{' '}
                <Text style={{ color: '#dcdcaa' }}>greeting</Text>(
                <Text style={{ color: '#9cdcfe' }}>name</Text>:{' '}
                <Text style={{ color: '#4ec9b0' }}>string</Text>)
              </div>
              <div style={{ color: '#8c8c8c' }}>
                3 <Text style={{ color: '#569cd6' }}>return</Text>{' '}
                <Text style={{ color: '#ce9178' }}>`Hello, {'${name}'}!`</Text>;
              </div>
              <div style={{ color: '#8c8c8c' }}>4</div>
              <div style={{ color: '#8c8c8c' }}>5</div>
              <div style={{ color: '#8c8c8c' }}>
                6 <Text style={{ color: '#6a9955' }}>// Call the function</Text>
              </div>
              <div style={{ color: '#8c8c8c' }}>
                7 <Text style={{ color: '#569cd6' }}>const</Text>{' '}
                <Text style={{ color: '#ffffff' }}>message</Text> ={' '}
                <Text style={{ color: '#dcdcaa' }}>greeting</Text>(
                <Text style={{ color: '#ce9178' }}>'Jan'</Text>);
              </div>
              <div style={{ color: '#8c8c8c' }}>
                8 <Text style={{ color: '#9cdcfe' }}>console</Text>.
                <Text style={{ color: '#dcdcaa' }}>log</Text>(
                <Text style={{ color: '#ffffff' }}>message</Text>);{' '}
                <Text style={{ color: '#6a9955' }}>
                  // Outputs: Hello, Jan!
                </Text>
              </div>
            </div>
          </Card>

          <Divider style={{ margin: 0 }} />

          <Flex justify="space-between" align="center">
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

          <Flex justify="space-between" align="center">
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
