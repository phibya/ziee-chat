import { Avatar, Button, Card, List, Space, Tag, Typography } from 'antd'
import { AppstoreOutlined, DownloadOutlined } from '@ant-design/icons'

const { Title, Text } = Typography

const mockModels = [
  {
    id: 'gpt-3.5-turbo',
    name: 'GPT-3.5 Turbo',
    description: 'Fast and efficient model for general conversations',
    provider: 'OpenAI',
    size: '~1.5GB',
    capabilities: ['Chat', 'Function Calling'],
    status: 'available',
  },
  {
    id: 'gpt-4',
    name: 'GPT-4',
    description: 'Most capable model for complex reasoning tasks',
    provider: 'OpenAI',
    size: '~3GB',
    capabilities: ['Chat', 'Function Calling', 'Vision'],
    status: 'available',
  },
  {
    id: 'claude-3-sonnet',
    name: 'Claude 3 Sonnet',
    description: 'Balanced model for various tasks',
    provider: 'Anthropic',
    size: '~2GB',
    capabilities: ['Chat', 'Function Calling', 'Vision'],
    status: 'available',
  },
  {
    id: 'llama-2-7b',
    name: 'Llama 2 7B',
    description: 'Open-source model for local deployment',
    provider: 'Meta',
    size: '~3.5GB',
    capabilities: ['Chat'],
    status: 'downloadable',
  },
  {
    id: 'mistral-7b',
    name: 'Mistral 7B',
    description: 'High-performance open-source model',
    provider: 'Mistral AI',
    size: '~3.8GB',
    capabilities: ['Chat', 'Function Calling'],
    status: 'downloadable',
  },
]

export function HubPage() {
  const handleDownload = (modelId: string) => {
    console.log('Downloading model:', modelId)
    // In a real app, this would trigger model download
  }

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'available':
        return 'green'
      case 'downloadable':
        return 'blue'
      case 'downloading':
        return 'orange'
      case 'installed':
        return 'purple'
      default:
        return 'default'
    }
  }

  return (
    <div style={{ padding: '24px', height: '100%', overflow: 'auto' }}>
      <div style={{ marginBottom: '24px' }}>
        <Title level={2}>Model Hub</Title>
        <Text type="secondary">
          Discover and download AI models for your local deployment
        </Text>
      </div>

      <List
        grid={{ gutter: 16, xs: 1, sm: 1, md: 2, lg: 2, xl: 3 }}
        dataSource={mockModels}
        renderItem={model => (
          <List.Item>
            <Card
              actions={[
                <Button
                  key="download"
                  type={model.status === 'available' ? 'default' : 'primary'}
                  icon={<DownloadOutlined />}
                  onClick={() => handleDownload(model.id)}
                  disabled={model.status === 'downloading'}
                >
                  {model.status === 'available' ? 'Use' : 'Download'}
                </Button>,
              ]}
              hoverable
            >
              <Card.Meta
                avatar={<Avatar size="large" icon={<AppstoreOutlined />} />}
                title={
                  <Space>
                    {model.name}
                    <Tag color={getStatusColor(model.status)}>
                      {model.status}
                    </Tag>
                  </Space>
                }
                description={
                  <div>
                    <Text style={{ marginBottom: '8px', display: 'block' }}>
                      {model.description}
                    </Text>
                    <div style={{ marginBottom: '8px' }}>
                      <Text type="secondary">Provider: </Text>
                      <Text strong>{model.provider}</Text>
                    </div>
                    <div style={{ marginBottom: '8px' }}>
                      <Text type="secondary">Size: </Text>
                      <Text code>{model.size}</Text>
                    </div>
                    <div>
                      <Text type="secondary">Capabilities: </Text>
                      <Space wrap>
                        {model.capabilities.map(capability => (
                          <Tag key={capability}>{capability}</Tag>
                        ))}
                      </Space>
                    </div>
                  </div>
                }
              />
            </Card>
          </List.Item>
        )}
      />
    </div>
  )
}
