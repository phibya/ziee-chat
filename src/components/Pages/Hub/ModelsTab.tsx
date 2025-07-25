import {
  AppstoreOutlined,
  ClearOutlined,
  DownloadOutlined,
  EyeOutlined,
  LockOutlined,
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
  Tag,
  Typography,
} from 'antd'
import { useMemo, useState } from 'react'
import { useHubStore, searchModels } from '../../../store/hub'

const { Title, Text } = Typography

export function ModelsTab() {
  const { models } = useHubStore()
  const { message } = App.useApp()
  const [searchTerm, setSearchTerm] = useState('')
  const [selectedTags, setSelectedTags] = useState<string[]>([])
  const [selectedCapabilities, setSelectedCapabilities] = useState<string[]>([])
  const [sortBy, setSortBy] = useState('popular')

  const handleDownload = async (model: HubModel) => {
    console.log('Downloading model:', model.id)
    message.info(`Starting download of ${model.name}`)
    // TODO: Implement actual model download
  }

  const clearAllFilters = () => {
    setSearchTerm('')
    setSelectedTags([])
    setSelectedCapabilities([])
  }

  // Get unique tags and capabilities for filters
  const modelTags = useMemo(() => {
    const allTags = new Set<string>()
    models.forEach(model => {
      model.tags.forEach(tag => allTags.add(tag))
    })
    return Array.from(allTags).sort()
  }, [models])

  const modelCapabilities = useMemo(() => {
    const capabilities: { key: string; label: string }[] = []
    const hasVision = models.some(m => m.capabilities?.vision)
    const hasTools = models.some(m => m.capabilities?.tools)
    const hasCode = models.some(m => m.capabilities?.code_interpreter)
    const hasAudio = models.some(m => m.capabilities?.audio)

    if (hasVision) capabilities.push({ key: 'vision', label: 'Vision' })
    if (hasTools) capabilities.push({ key: 'tools', label: 'Tools' })
    if (hasCode) capabilities.push({ key: 'code_interpreter', label: 'Code Interpreter' })
    if (hasAudio) capabilities.push({ key: 'audio', label: 'Audio' })

    return capabilities
  }, [models])

  const filteredModels = useMemo(() => {
    let filtered = searchModels(models, searchTerm)

    // Filter by tags
    if (selectedTags.length > 0) {
      filtered = filtered.filter(model => 
        selectedTags.some(tag => model.tags.includes(tag))
      )
    }

    // Filter by capabilities
    if (selectedCapabilities.length > 0) {
      filtered = filtered.filter(model => {
        if (!model.capabilities) return false
        return selectedCapabilities.some(capability => {
          switch (capability) {
            case 'vision':
              return model.capabilities?.vision || false
            case 'tools':
              return model.capabilities?.tools || false
            case 'code_interpreter':
              return model.capabilities?.code_interpreter || false
            case 'audio':
              return model.capabilities?.audio || false
            default:
              return false
          }
        })
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
  }, [models, searchTerm, selectedTags, selectedCapabilities, sortBy])

  return (
    <>
      {/* Search and Filters */}
      <div className="mb-6">
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
              mode="multiple"
              placeholder="Filter by tags"
              value={selectedTags}
              onChange={setSelectedTags}
              style={{ width: '100%' }}
              allowClear
              maxTagCount="responsive"
            >
              {modelTags.map(tag => (
                <Select.Option key={tag} value={tag}>
                  {tag}
                </Select.Option>
              ))}
            </Select>
          </Col>
          <Col xs={12} sm={12} md={6} lg={4}>
            <Select
              mode="multiple"
              placeholder="Capabilities"
              value={selectedCapabilities}
              onChange={setSelectedCapabilities}
              style={{ width: '100%' }}
              allowClear
              maxTagCount="responsive"
            >
              {modelCapabilities.map(capability => (
                <Select.Option key={capability.key} value={capability.key}>
                  {capability.label}
                </Select.Option>
              ))}
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
              <Select.Option value="size">Size</Select.Option>
            </Select>
          </Col>
          {(searchTerm || selectedTags.length > 0 || selectedCapabilities.length > 0) && (
            <Col xs={24} sm={24} md={12} lg={8}>
              <Flex align="center" gap={8}>
                <Text type="secondary" style={{ fontSize: '12px' }}>
                  Filters active: {[
                    searchTerm && 'search',
                    selectedTags.length > 0 && `${selectedTags.length} tags`,
                    selectedCapabilities.length > 0 && `${selectedCapabilities.length} capabilities`
                  ].filter(Boolean).join(', ')}
                </Text>
                <Button
                  size="small"
                  type="text"
                  icon={<ClearOutlined />}
                  onClick={clearAllFilters}
                >
                  Clear all
                </Button>
              </Flex>
            </Col>
          )}
        </Row>
      </div>

      {/* Models Grid */}
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
  )
}