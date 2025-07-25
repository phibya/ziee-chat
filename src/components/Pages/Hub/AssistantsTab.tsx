import { ClearOutlined, RobotOutlined, SearchOutlined } from '@ant-design/icons'
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
import { useHubStore, searchAssistants } from '../../../store/hub'
import type { HubAssistant } from '../../../types/api/hub'

const { Title, Text } = Typography

export function AssistantsTab() {
  const { assistants } = useHubStore()
  const { message } = App.useApp()
  const [searchTerm, setSearchTerm] = useState('')
  const [selectedTags, setSelectedTags] = useState<string[]>([])
  const [sortBy, setSortBy] = useState('popular')

  const handleUseAssistant = (assistant: HubAssistant) => {
    console.log('Using assistant:', assistant.id)
    message.info(`Starting conversation with ${assistant.name}`)
    // TODO: Navigate to chat with assistant
  }

  const clearAllFilters = () => {
    setSearchTerm('')
    setSelectedTags([])
  }

  // Get unique tags for filters
  const assistantTags = useMemo(() => {
    const allTags = new Set<string>()
    assistants.forEach(assistant => {
      assistant.tags.forEach(tag => allTags.add(tag))
    })
    return Array.from(allTags).sort()
  }, [assistants])

  const filteredAssistants = useMemo(() => {
    let filtered = searchAssistants(assistants, searchTerm)

    // Filter by tags
    if (selectedTags.length > 0) {
      filtered = filtered.filter(assistant =>
        selectedTags.some(tag => assistant.tags.includes(tag)),
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
  }, [assistants, searchTerm, selectedTags, sortBy])

  return (
    <>
      {/* Search and Filters */}
      <div className="mb-6">
        <Row gutter={[16, 16]}>
          <Col xs={24} sm={24} md={12} lg={8}>
            <Input
              placeholder="Search assistants..."
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
              {assistantTags.map(tag => (
                <Select.Option key={tag} value={tag}>
                  {tag}
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
            </Select>
          </Col>
          {(searchTerm || selectedTags.length > 0) && (
            <Col xs={24} sm={24} md={12} lg={8}>
              <Flex align="center" gap={8}>
                <Text type="secondary" style={{ fontSize: '12px' }}>
                  Filters active:{' '}
                  {[
                    searchTerm && 'search',
                    selectedTags.length > 0 && `${selectedTags.length} tags`,
                  ]
                    .filter(Boolean)
                    .join(', ')}
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

      {/* Assistants Grid */}
      <Row gutter={[16, 16]}>
        {filteredAssistants.map(assistant => (
          <Col xs={24} sm={24} md={12} lg={8} xl={6} key={assistant.id}>
            <Card
              hoverable
              style={{ height: '100%' }}
              styles={{ body: { padding: '16px' } }}
            >
              <div style={{ marginBottom: '12px' }}>
                <Title level={4} style={{ margin: 0, marginBottom: '4px' }}>
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
                    <Tag key={tag} color="default" style={{ fontSize: '11px' }}>
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
  )
}
