import { ClearOutlined, SearchOutlined } from '@ant-design/icons'
import { App, Button, Flex, Input, Select, Spin, Typography } from 'antd'
import { useMemo, useState } from 'react'
import { searchAssistants, loadHubAssistants } from '../../../store/hub'
import { AssistantCard } from './AssistantCard'
import { useMainContentMinSize } from '../../hooks/useWindowMinSize.ts'
import { VscFilter } from 'react-icons/vsc'
import { Stores } from '../../../store'

const { Text } = Typography

export function AssistantsTab() {
  const { message } = App.useApp()
  const {
    assistants,
    assistantsInitialized,
    assistantsLoading,
    assistantsError,
  } = Stores.Hub
  const [searchTerm, setSearchTerm] = useState('')
  const [selectedTags, setSelectedTags] = useState<string[]>([])
  const [sortBy, setSortBy] = useState('popular')
  const mainContentMinSize = useMainContentMinSize()
  const [showFilters, setShowFilters] = useState(false)

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

  const filters = (
    <>
      <Select
        mode="multiple"
        placeholder="Filter by tags"
        value={selectedTags}
        onChange={setSelectedTags}
        className="flex-1"
        allowClear
        maxTagCount="responsive"
        options={assistantTags.map(tag => ({
          key: tag,
          value: tag,
          label: tag,
        }))}
        popupMatchSelectWidth={false}
      />
      <Select
        placeholder="Sort by"
        value={sortBy}
        onChange={setSortBy}
        className="flex-1"
        options={[
          { value: 'popular', label: 'Popular' },
          { value: 'name', label: 'Name' },
        ]}
        popupMatchSelectWidth={false}
      />
    </>
  )

  const toolbar = (
    <div className="flex gap-2 flex-wrap">
      <div className={'flex gap-2 w-full'}>
        <Input
          placeholder="Search assistants..."
          prefix={<SearchOutlined />}
          value={searchTerm}
          onChange={e => setSearchTerm(e.target.value)}
          allowClear
          className="flex-1"
        />
        {!mainContentMinSize.xs ? (
          filters
        ) : (
          <Button
            type={showFilters ? 'primary' : 'default'}
            className={'!text-lg'}
            onClick={() => setShowFilters(!showFilters)}
          >
            <VscFilter />
          </Button>
        )}
      </div>
      {mainContentMinSize.xs && showFilters && (
        <div className={'flex gap-2 w-full'}>{filters}</div>
      )}
    </div>
  )

  // Show loading state
  if (assistantsLoading && !assistantsInitialized) {
    return (
      <div className="flex justify-center items-center h-full">
        <Spin size="large" />
        <Text className="ml-4">Loading assistants...</Text>
      </div>
    )
  }

  // Show error state
  if (assistantsError && !assistantsInitialized) {
    return (
      <div className="text-center py-12">
        <Text type="danger">Failed to load assistants: {assistantsError}</Text>
        <div className="mt-4">
          <Button
            onClick={() => {
              loadHubAssistants().catch(err => {
                console.error('Failed to load hub assistants:', err)
                message.error('Failed to load hub assistants')
              })
            }}
          >
            Retry
          </Button>
        </div>
      </div>
    )
  }

  return (
    <div className="flex flex-col gap-3 h-full overflow-hidden">
      {/* Search and Filters */}
      <div className={'px-3'}>
        {toolbar}
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

      {/* Assistants List */}
      <div className="flex-1 overflow-auto px-3 pb-3">
        <div className="flex flex-col gap-3">
          {filteredAssistants.map(assistant => (
            <AssistantCard key={assistant.id} assistant={assistant} />
          ))}
        </div>

        {filteredAssistants.length === 0 && (
          <div className="text-center py-12">
            <Text type="secondary">No assistants found</Text>
          </div>
        )}
      </div>
    </div>
  )
}
