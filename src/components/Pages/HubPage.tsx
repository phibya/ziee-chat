import {
  Button,
  Card,
  Input,
  Space,
  Tag,
  Typography,
  Select,
  Row,
  Col,
} from 'antd'
import {
  DownloadOutlined,
  SearchOutlined,
  ClockCircleOutlined,
  TagOutlined,
} from '@ant-design/icons'
import { useState, useMemo } from 'react'

const { Title, Text } = Typography

const mockModels = [
  {
    id: 'llama3.1-8b',
    name: 'llama3.1',
    description:
      'Llama 3.1 is a new state-of-the-art model from Meta available in 8B, 70B and 405B parameter sizes.',
    provider: 'Meta',
    size: '8b',
    variants: ['8b', '70b', '405b'],
    capabilities: ['tools'],
    status: 'available',
    pulls: '97.4M',
    tags: 93,
    lastUpdated: '7 months ago',
  },
  {
    id: 'llama3.2-1b',
    name: 'llama3.2',
    description: "Meta's Llama 3.2 goes small with 1B and 3B models.",
    provider: 'Meta',
    size: '1b',
    variants: ['1b', '3b'],
    capabilities: ['tools'],
    status: 'available',
    pulls: '24.2M',
    tags: 63,
    lastUpdated: '9 months ago',
  },
  {
    id: 'llama3-70b',
    name: 'llama3',
    description: 'Meta Llama 3: The most capable openly available LLM to date',
    provider: 'Meta',
    size: '70b',
    variants: ['8b', '70b'],
    capabilities: [],
    status: 'available',
    pulls: '9.2M',
    tags: 68,
    lastUpdated: '1 year ago',
  },
  {
    id: 'llava-7b',
    name: 'llava',
    description:
      '🌋 LLaVA is a novel end-to-end trained large multimodal model that combines a vision encoder and Vicuna for general-purpose visual and language understanding. Updated to version 1.6.',
    provider: 'LLaVA',
    size: '7b',
    variants: ['7b', '13b', '34b'],
    capabilities: ['vision'],
    status: 'available',
    pulls: '7.5M',
    tags: 98,
    lastUpdated: '1 year ago',
  },
  {
    id: 'llama2-7b',
    name: 'llama2',
    description:
      'Llama 2 is a collection of foundation language models ranging from 7B to 70B parameters.',
    provider: 'Meta',
    size: '7b',
    variants: ['7b', '13b', '70b'],
    capabilities: [],
    status: 'available',
    pulls: '3.8M',
    tags: 102,
    lastUpdated: '1 year ago',
  },
  {
    id: 'gpt-3.5-turbo',
    name: 'GPT-3.5 Turbo',
    description: 'Fast and efficient model for general conversations',
    provider: 'OpenAI',
    size: '~1.5GB',
    variants: [],
    capabilities: ['Chat', 'Function Calling'],
    status: 'available',
    pulls: '45.2M',
    tags: 12,
    lastUpdated: '3 months ago',
  },
  {
    id: 'gpt-4',
    name: 'GPT-4',
    description: 'Most capable model for complex reasoning tasks',
    provider: 'OpenAI',
    size: '~3GB',
    variants: [],
    capabilities: ['Chat', 'Function Calling', 'Vision'],
    status: 'available',
    pulls: '32.1M',
    tags: 8,
    lastUpdated: '2 months ago',
  },
  {
    id: 'claude-3-sonnet',
    name: 'Claude 3 Sonnet',
    description: 'Balanced model for various tasks',
    provider: 'Anthropic',
    size: '~2GB',
    variants: [],
    capabilities: ['Chat', 'Function Calling', 'Vision'],
    status: 'available',
    pulls: '28.7M',
    tags: 15,
    lastUpdated: '4 months ago',
  },
]

export function HubPage() {
  const [searchTerm, setSearchTerm] = useState('')
  const [selectedCategory, setSelectedCategory] = useState('all')
  const [sortBy, setSortBy] = useState('popular')

  const handleDownload = (modelId: string) => {
    console.log('Downloading model:', modelId)
    // In a real app, this would trigger model download
  }

  const getCapabilityColor = (capability: string) => {
    switch (capability.toLowerCase()) {
      case 'tools':
        return 'blue'
      case 'vision':
        return 'purple'
      case 'chat':
        return 'green'
      case 'function calling':
        return 'orange'
      default:
        return 'default'
    }
  }

  const filteredModels = useMemo(() => {
    let filtered = mockModels.filter(model => {
      const matchesSearch =
        model.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
        model.description.toLowerCase().includes(searchTerm.toLowerCase())
      const matchesCategory =
        selectedCategory === 'all' ||
        model.capabilities.some(cap =>
          cap.toLowerCase().includes(selectedCategory.toLowerCase()),
        )
      return matchesSearch && matchesCategory
    })

    // Sort models
    switch (sortBy) {
      case 'popular':
        filtered.sort(
          (a, b) =>
            parseFloat(b.pulls.replace(/[M]/g, '')) -
            parseFloat(a.pulls.replace(/[M]/g, '')),
        )
        break
      case 'name':
        filtered.sort((a, b) => a.name.localeCompare(b.name))
        break
      case 'updated':
        // Simple sorting by last updated text - in real app would use actual dates
        filtered.sort((a, b) => a.lastUpdated.localeCompare(b.lastUpdated))
        break
    }

    return filtered
  }, [searchTerm, selectedCategory, sortBy])

  return (
    <div style={{ padding: '24px', height: '100%', overflow: 'auto' }}>
      {/* Header */}
      <div style={{ marginBottom: '24px' }}>
        <Title level={2} style={{ marginBottom: '8px' }}>
          Model Hub
        </Title>
        <Text type="secondary">
          Discover and download AI models for your local deployment
        </Text>
      </div>

      {/* Search and Filters */}
      <div
        style={{
          marginBottom: '24px',
          background: '#fafafa',
          padding: '16px',
          borderRadius: '8px',
        }}
      >
        <Row gutter={[16, 16]}>
          <Col xs={24} sm={24} md={12} lg={8}>
            <Input
              placeholder="Search models..."
              prefix={<SearchOutlined />}
              value={searchTerm}
              onChange={e => setSearchTerm(e.target.value)}
              allowClear
            />
          </Col>
          <Col xs={12} sm={12} md={6} lg={4}>
            <Select
              placeholder="Category"
              value={selectedCategory}
              onChange={setSelectedCategory}
              style={{ width: '100%' }}
            >
              <Select.Option value="all">All</Select.Option>
              <Select.Option value="embedding">Embedding</Select.Option>
              <Select.Option value="vision">Vision</Select.Option>
              <Select.Option value="tools">Tools</Select.Option>
              <Select.Option value="thinking">Thinking</Select.Option>
            </Select>
          </Col>
          <Col xs={12} sm={12} md={6} lg={4}>
            <Select
              placeholder="Sort by"
              value={sortBy}
              onChange={setSortBy}
              style={{ width: '100%' }}
            >
              <Select.Option value="popular">Popular</Select.Option>
              <Select.Option value="name">Name</Select.Option>
              <Select.Option value="updated">Recently Updated</Select.Option>
            </Select>
          </Col>
        </Row>
      </div>

      {/* Model Cards */}
      <Row gutter={[16, 16]}>
        {filteredModels.map(model => (
          <Col xs={24} sm={24} md={12} lg={8} xl={6} key={model.id}>
            <Card
              hoverable
              style={{ height: '100%' }}
              bodyStyle={{ padding: '16px' }}
            >
              <div style={{ marginBottom: '12px' }}>
                <Title level={4} style={{ margin: 0, marginBottom: '4px' }}>
                  {model.name}
                </Title>
                <Text type="secondary" style={{ fontSize: '12px' }}>
                  {model.description}
                </Text>
              </div>

              {/* Variants/Tags */}
              {model.variants && model.variants.length > 0 && (
                <div style={{ marginBottom: '12px' }}>
                  <Space wrap>
                    {model.variants.map(variant => (
                      <Tag
                        key={variant}
                        color="blue"
                        style={{ fontSize: '11px' }}
                      >
                        {variant}
                      </Tag>
                    ))}
                  </Space>
                </div>
              )}

              {/* Capabilities */}
              {model.capabilities && model.capabilities.length > 0 && (
                <div style={{ marginBottom: '12px' }}>
                  <Space wrap>
                    {model.capabilities.map(capability => (
                      <Tag
                        key={capability}
                        color={getCapabilityColor(capability)}
                        style={{ fontSize: '11px' }}
                      >
                        {capability}
                      </Tag>
                    ))}
                  </Space>
                </div>
              )}

              {/* Stats */}
              <div style={{ marginBottom: '12px' }}>
                <div
                  style={{
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'center',
                    marginBottom: '4px',
                  }}
                >
                  <Space size={4}>
                    <DownloadOutlined
                      style={{ fontSize: '12px', color: '#666' }}
                    />
                    <Text type="secondary" style={{ fontSize: '12px' }}>
                      {model.pulls} Pulls
                    </Text>
                  </Space>
                  <Space size={4}>
                    <TagOutlined style={{ fontSize: '12px', color: '#666' }} />
                    <Text type="secondary" style={{ fontSize: '12px' }}>
                      {model.tags} Tags
                    </Text>
                  </Space>
                </div>
                <div style={{ display: 'flex', alignItems: 'center' }}>
                  <ClockCircleOutlined
                    style={{
                      fontSize: '12px',
                      color: '#666',
                      marginRight: '4px',
                    }}
                  />
                  <Text type="secondary" style={{ fontSize: '12px' }}>
                    Updated {model.lastUpdated}
                  </Text>
                </div>
              </div>

              {/* Action Button */}
              <Button
                type="primary"
                block
                size="small"
                onClick={() => handleDownload(model.id)}
                style={{ marginTop: 'auto' }}
              >
                Pull Model
              </Button>
            </Card>
          </Col>
        ))}
      </Row>

      {filteredModels.length === 0 && (
        <div style={{ textAlign: 'center', padding: '48px' }}>
          <Text type="secondary">No models found matching your criteria.</Text>
        </div>
      )}
    </div>
  )
}
