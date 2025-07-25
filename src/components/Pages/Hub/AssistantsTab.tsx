import { ClearOutlined, RobotOutlined, SearchOutlined } from '@ant-design/icons'
import {
  App,
  Button,
  Card,
  Flex,
  Input,
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
        <Flex wrap gap={16} className="mb-4">
          <div className="flex-1 min-w-[200px] basis-[300px]">
            <Input
              placeholder="Search assistants..."
              prefix={<SearchOutlined />}
              value={searchTerm}
              onChange={e => setSearchTerm(e.target.value)}
              allowClear
            />
          </div>
          <div className="flex-1 min-w-[150px] basis-[200px]">
            <Select
              mode="multiple"
              placeholder="Filter by tags"
              value={selectedTags}
              onChange={setSelectedTags}
              className="w-full"
              allowClear
              maxTagCount="responsive"
            >
              {assistantTags.map(tag => (
                <Select.Option key={tag} value={tag}>
                  {tag}
                </Select.Option>
              ))}
            </Select>
          </div>
          <div className="flex-1 min-w-[120px] basis-[150px]">
            <Select
              placeholder="Sort by"
              value={sortBy}
              onChange={setSortBy}
              className="w-full"
            >
              <Select.Option value="popular">Popular</Select.Option>
              <Select.Option value="name">Name</Select.Option>
            </Select>
          </div>
        </Flex>
        {(searchTerm || selectedTags.length > 0) && (
          <Flex align="center" gap={8}>
            <Text type="secondary" className="text-xs">
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
        )}
      </div>

      {/* Assistants Grid */}
      <div className="grid grid-cols-[repeat(auto-fill,minmax(300px,1fr))] gap-4">
        {filteredAssistants.map(assistant => (
          <Card
            key={assistant.id}
            hoverable
            className="h-full"
            styles={{ body: { padding: '16px' } }}
          >
            <div className="mb-3">
              <Title level={4} className="m-0 mb-1">
                {assistant.name}
              </Title>
              <Text type="secondary" className="text-xs">
                {assistant.description}
              </Text>
            </div>

            {/* Category & Author */}
            <div className="mb-3">
              <Flex justify="space-between" align="center">
                <Tag color="geekblue" className="text-xs">
                  {assistant.category}
                </Tag>
                {assistant.author && (
                  <Text type="secondary" className="text-xs">
                    by {assistant.author}
                  </Text>
                )}
              </Flex>
            </div>

            {/* Tags */}
            <div className="mb-3">
              <Flex wrap className="gap-1">
                {assistant.tags.slice(0, 3).map(tag => (
                  <Tag key={tag} color="default" className="text-xs">
                    {tag}
                  </Tag>
                ))}
                {assistant.tags.length > 3 && (
                  <Tag color="default" className="text-xs">
                    +{assistant.tags.length - 3}
                  </Tag>
                )}
              </Flex>
            </div>

            {/* Recommended Models */}
            {assistant.recommended_models.length > 0 && (
              <div className="mb-3">
                <Text type="secondary" className="text-xs">
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
              className="mt-auto"
            >
              Use Assistant
            </Button>
          </Card>
        ))}
      </div>

      {filteredAssistants.length === 0 && (
        <div className="text-center py-12">
          <Text type="secondary">No assistants found</Text>
        </div>
      )}
    </>
  )
}
