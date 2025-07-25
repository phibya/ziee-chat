import {
  AppstoreOutlined,
  DownloadOutlined,
  EyeOutlined,
  LockOutlined,
  ReloadOutlined,
  RobotOutlined,
  SearchOutlined,
  ToolOutlined,
  UnlockOutlined,
} from '@ant-design/icons'
import {
  App,
  Button,
  Card,
  Col,
  Flex,
  Input,
  Row,
  Select,
  Spin,
  Tag,
  Tabs,
  Typography,
} from 'antd'
import { useEffect, useMemo, useState } from 'react'
import { PageContainer } from '../common/PageContainer'
import {
  useHubStore,
  initializeHub,
  refreshHub,
  searchModels,
  searchAssistants,
} from '../../store/hub'
import type { HubModel, HubAssistant } from '../../api/hub'

const { Title, Text } = Typography

export function HubPage() {
  const { message } = App.useApp()
  const [searchTerm, setSearchTerm] = useState('')
  const [selectedCategory, setSelectedCategory] = useState('all')
  const [sortBy, setSortBy] = useState('popular')
  const [activeTab, setActiveTab] = useState('models')

  // Hub store state
  const {
    models,
    assistants,
    hubVersion,
    lastUpdated,
    initialized,
    loading,
    error,
  } = useHubStore()

  useEffect(() => {
    if (!initialized && !loading && !error) {
      initializeHub().catch(err => {
        console.error('Failed to initialize hub:', err)
        message.error('Failed to load hub data')
      })
    }
  }, [initialized, loading, error, message])

  const handleDownload = async (model: HubModel) => {
    console.log('Downloading model:', model.id)
    message.info(`Starting download of ${model.name}`)
    // TODO: Implement actual model download
  }

  const handleUseAssistant = (assistant: HubAssistant) => {
    console.log('Using assistant:', assistant.id)
    message.info(`Starting conversation with ${assistant.name}`)
    // TODO: Navigate to chat with assistant
  }

  const handleRefresh = async () => {
    try {
      await refreshHub()
      message.success('Hub data refreshed successfully')
    } catch (err) {
      console.error('Failed to refresh hub:', err)
      message.error('Failed to refresh hub data')
    }
  }

  const filteredModels = useMemo(() => {
    let filtered = searchModels(models, searchTerm)

    // Filter by category
    if (selectedCategory !== 'all') {
      filtered = filtered.filter(model => {
        if (!model.capabilities) return false
        switch (selectedCategory) {
          case 'vision':
            return model.capabilities.vision
          case 'tools':
            return model.capabilities.tools
          case 'code':
            return model.capabilities.code_interpreter
          default:
            return true
        }
      })
    }

    // Sort models
    switch (sortBy) {
      case 'popular':
        filtered.sort(
          (a, b) => (b.popularity_score || 0) - (a.popularity_score || 0),
        )
        break
      case 'name':
        filtered.sort((a, b) => a.name.localeCompare(b.name))
        break
      case 'size':
        filtered.sort((a, b) => a.size_gb - b.size_gb)
        break
      default:
        break
    }

    return filtered
  }, [models, searchTerm, selectedCategory, sortBy])

  const filteredAssistants = useMemo(() => {
    let filtered = searchAssistants(assistants, searchTerm)

    // Filter by category
    if (selectedCategory !== 'all') {
      filtered = filtered.filter(
        assistant =>
          assistant.category.toLowerCase() === selectedCategory.toLowerCase(),
      )
    }

    // Sort assistants
    switch (sortBy) {
      case 'popular':
        filtered.sort(
          (a, b) => (b.popularity_score || 0) - (a.popularity_score || 0),
        )
        break
      case 'name':
        filtered.sort((a, b) => a.name.localeCompare(b.name))
        break
      default:
        break
    }

    return filtered
  }, [assistants, searchTerm, selectedCategory, sortBy])

  if (loading && !initialized) {
    return (
      <PageContainer>
        <div className="flex justify-center items-center h-64">
          <Spin size="large" />
          <Text className="ml-4">Loading hub data...</Text>
        </div>
      </PageContainer>
    )
  }

  if (error && !initialized) {
    return (
      <PageContainer>
        <div className="text-center py-12">
          <Text type="danger">Failed to load hub data: {error}</Text>
          <div className="mt-4">
            <Button
              onClick={() => {
                // Clear error and retry
                useHubStore.setState({ error: null })
                initializeHub()
              }}
            >
              Retry
            </Button>
          </div>
        </div>
      </PageContainer>
    )
  }

  return (
    <PageContainer>
      <div style={{ height: '100%', overflow: 'auto' }}>
        {/* Header */}
        <div className="mb-6">
          <Flex justify="space-between" align="center" className="mb-2">
            <Title level={2} style={{ margin: 0 }}>
              Hub
            </Title>
            <Flex align="center" gap={16}>
              <Text type="secondary" className="text-sm">
                Version: {hubVersion} • Updated:{' '}
                {new Date(lastUpdated).toLocaleDateString()}
              </Text>
              <Button
                icon={<ReloadOutlined />}
                onClick={handleRefresh}
                loading={loading}
                type="text"
              >
                Refresh
              </Button>
            </Flex>
          </Flex>
          <Text type="secondary">
            Discover and download models and assistants
          </Text>
        </div>

        {/* Tabs */}
        <Tabs
          activeKey={activeTab}
          onChange={setActiveTab}
          className="mb-6"
          items={[
            {
              key: 'models',
              label: (
                <span>
                  <AppstoreOutlined />
                  Models ({models.length})
                </span>
              ),
              children: null,
            },
            {
              key: 'assistants',
              label: (
                <span>
                  <RobotOutlined />
                  Assistants ({assistants.length})
                </span>
              ),
              children: null,
            },
          ]}
        />

        {/* Search and Filters */}
        <div className="mb-6">
          <Row gutter={[16, 16]}>
            <Col xs={24} sm={24} md={12} lg={8}>
              <Input
                placeholder={`Search ${activeTab}...`}
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
                {activeTab === 'models' ? (
                  <>
                    <Select.Option value="vision">Vision</Select.Option>
                    <Select.Option value="tools">Tools</Select.Option>
                    <Select.Option value="code">Code</Select.Option>
                  </>
                ) : (
                  <>
                    <Select.Option value="development">
                      Development
                    </Select.Option>
                    <Select.Option value="writing">Writing</Select.Option>
                    <Select.Option value="vision">Vision</Select.Option>
                    <Select.Option value="analytics">Analytics</Select.Option>
                  </>
                )}
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
                {activeTab === 'models' && (
                  <Select.Option value="size">Size</Select.Option>
                )}
              </Select>
            </Col>
          </Row>
        </div>

        {/* Content */}
        {activeTab === 'models' ? (
          <>
            <Row gutter={[16, 16]}>
              {filteredModels.map(model => (
                <Col xs={24} sm={24} md={12} lg={8} xl={6} key={model.id}>
                  <Card
                    hoverable
                    style={{ height: '100%' }}
                    styles={{ body: { padding: '16px' } }}
                  >
                    <div style={{ marginBottom: '12px' }}>
                      <Flex
                        justify="space-between"
                        align="start"
                        className="mb-2"
                      >
                        <Title level={4} style={{ margin: 0 }}>
                          {model.alias}
                        </Title>
                        {model.public ? (
                          <UnlockOutlined style={{ color: '#52c41a' }} />
                        ) : (
                          <LockOutlined style={{ color: '#ff4d4f' }} />
                        )}
                      </Flex>
                      <Text type="secondary" style={{ fontSize: '12px' }}>
                        {model.description}
                      </Text>
                    </div>

                    {/* Tags */}
                    <div style={{ marginBottom: '12px' }}>
                      <Flex wrap className="gap-1">
                        {model.tags.slice(0, 3).map(tag => (
                          <Tag
                            key={tag}
                            color="default"
                            style={{ fontSize: '11px' }}
                          >
                            {tag}
                          </Tag>
                        ))}
                        {model.tags.length > 3 && (
                          <Tag color="default" style={{ fontSize: '11px' }}>
                            +{model.tags.length - 3}
                          </Tag>
                        )}
                      </Flex>
                    </div>

                    {/* Capabilities */}
                    {model.capabilities && (
                      <div style={{ marginBottom: '12px' }}>
                        <Flex wrap className="gap-1">
                          {model.capabilities.vision && (
                            <Tag
                              color="purple"
                              icon={<EyeOutlined />}
                              style={{ fontSize: '11px' }}
                            >
                              Vision
                            </Tag>
                          )}
                          {model.capabilities.tools && (
                            <Tag
                              color="blue"
                              icon={<ToolOutlined />}
                              style={{ fontSize: '11px' }}
                            >
                              Tools
                            </Tag>
                          )}
                          {model.capabilities.code_interpreter && (
                            <Tag
                              color="orange"
                              icon={<AppstoreOutlined />}
                              style={{ fontSize: '11px' }}
                            >
                              Code
                            </Tag>
                          )}
                        </Flex>
                      </div>
                    )}

                    {/* Stats */}
                    <div style={{ marginBottom: '12px' }}>
                      <Flex
                        justify="space-between"
                        align="center"
                        className="mb-1"
                      >
                        <Text type="secondary" style={{ fontSize: '12px' }}>
                          Size: {model.size_gb}GB
                        </Text>
                        <Text type="secondary" style={{ fontSize: '12px' }}>
                          {model.file_format.toUpperCase()}
                        </Text>
                      </Flex>
                      {model.license && (
                        <Text type="secondary" style={{ fontSize: '11px' }}>
                          License: {model.license}
                        </Text>
                      )}
                    </div>

                    {/* Action Button */}
                    <Button
                      type="primary"
                      block
                      size="small"
                      icon={<DownloadOutlined />}
                      onClick={() => handleDownload(model)}
                      style={{ marginTop: 'auto' }}
                    >
                      Download Model
                    </Button>
                  </Card>
                </Col>
              ))}
            </Row>

            {filteredModels.length === 0 && (
              <div className="text-center py-12">
                <Text type="secondary">No models found</Text>
              </div>
            )}
          </>
        ) : (
          <>
            <Row gutter={[16, 16]}>
              {filteredAssistants.map(assistant => (
                <Col xs={24} sm={24} md={12} lg={8} xl={6} key={assistant.id}>
                  <Card
                    hoverable
                    style={{ height: '100%' }}
                    styles={{ body: { padding: '16px' } }}
                  >
                    <div style={{ marginBottom: '12px' }}>
                      <Title
                        level={4}
                        style={{ margin: 0, marginBottom: '4px' }}
                      >
                        {assistant.name}
                      </Title>
                      <Text type="secondary" style={{ fontSize: '12px' }}>
                        {assistant.description}
                      </Text>
                    </div>

                    {/* Category & Author */}
                    <div style={{ marginBottom: '12px' }}>
                      <Flex justify="space-between" align="center">
                        <Tag color="geekblue" style={{ fontSize: '11px' }}>
                          {assistant.category}
                        </Tag>
                        {assistant.author && (
                          <Text type="secondary" style={{ fontSize: '11px' }}>
                            by {assistant.author}
                          </Text>
                        )}
                      </Flex>
                    </div>

                    {/* Tags */}
                    <div style={{ marginBottom: '12px' }}>
                      <Flex wrap className="gap-1">
                        {assistant.tags.slice(0, 3).map(tag => (
                          <Tag
                            key={tag}
                            color="default"
                            style={{ fontSize: '11px' }}
                          >
                            {tag}
                          </Tag>
                        ))}
                        {assistant.tags.length > 3 && (
                          <Tag color="default" style={{ fontSize: '11px' }}>
                            +{assistant.tags.length - 3}
                          </Tag>
                        )}
                      </Flex>
                    </div>

                    {/* Recommended Models */}
                    {assistant.recommended_models.length > 0 && (
                      <div style={{ marginBottom: '12px' }}>
                        <Text type="secondary" style={{ fontSize: '11px' }}>
                          Works best with:{' '}
                          {assistant.recommended_models.slice(0, 2).join(', ')}
                          {assistant.recommended_models.length > 2 && '...'}
                        </Text>
                      </div>
                    )}

                    {/* Action Button */}
                    <Button
                      type="primary"
                      block
                      size="small"
                      icon={<RobotOutlined />}
                      onClick={() => handleUseAssistant(assistant)}
                      style={{ marginTop: 'auto' }}
                    >
                      Use Assistant
                    </Button>
                  </Card>
                </Col>
              ))}
            </Row>

            {filteredAssistants.length === 0 && (
              <div className="text-center py-12">
                <Text type="secondary">No assistants found</Text>
              </div>
            )}
          </>
        )}
      </div>
    </PageContainer>
  )
}
