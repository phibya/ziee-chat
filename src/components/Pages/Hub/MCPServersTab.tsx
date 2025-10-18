import { ClearOutlined, SearchOutlined } from '@ant-design/icons'
import { App, Button, Flex, Input, Select, Spin, Typography } from 'antd'
import { useMemo, useState } from 'react'
import { searchMCPServers, loadHubMCPServers } from '../../../store/hub'
import { MCPServerCard } from './MCPServerCard'
import { Stores } from '../../../store'
import { useMainContentMinSize } from '../../hooks/useWindowMinSize.ts'
import { VscFilter } from 'react-icons/vsc'
import { isTauriView } from '../../../api/core'

const { Text } = Typography

export function MCPServersTab() {
  const { message } = App.useApp()
  const {
    mcpServers,
    mcpServersInitialized,
    mcpServersLoading,
    mcpServersError,
  } = Stores.Hub
  const [searchTerm, setSearchTerm] = useState('')
  const [selectedTags, setSelectedTags] = useState<string[]>([])
  const [selectedCategories, setSelectedCategories] = useState<string[]>([])
  const [selectedTransportTypes, setSelectedTransportTypes] = useState<
    string[]
  >([])
  const [sortBy, setSortBy] = useState('popular')
  const mainContentMinSize = useMainContentMinSize()
  const [showFilters, setShowFilters] = useState(false)
  const isDesktop = isTauriView

  const clearAllFilters = () => {
    setSearchTerm('')
    setSelectedTags([])
    setSelectedCategories([])
    setSelectedTransportTypes([])
  }

  // Get unique tags, categories, and transport types for filters
  const serverTags = useMemo(() => {
    const allTags = new Set<string>()
    mcpServers.forEach(server => {
      server.tags.forEach(tag => allTags.add(tag))
    })
    return Array.from(allTags).sort()
  }, [mcpServers])

  const serverCategories = useMemo(() => {
    const categories = new Set<string>()
    mcpServers.forEach(server => {
      categories.add(server.category)
    })
    return Array.from(categories).sort()
  }, [mcpServers])

  const transportTypes = useMemo(() => {
    const types = new Set<string>()
    mcpServers.forEach(server => {
      types.add(server.transport_type)
    })
    return Array.from(types).sort()
  }, [mcpServers])

  const filteredServers = useMemo(() => {
    let filtered = searchMCPServers(mcpServers, searchTerm)

    // Filter by tags
    if (selectedTags.length > 0) {
      filtered = filtered.filter(server =>
        selectedTags.some(tag => server.tags.includes(tag)),
      )
    }

    // Filter by categories
    if (selectedCategories.length > 0) {
      filtered = filtered.filter(server =>
        selectedCategories.includes(server.category),
      )
    }

    // Filter by transport types
    if (selectedTransportTypes.length > 0) {
      filtered = filtered.filter(server =>
        selectedTransportTypes.includes(server.transport_type),
      )
    }

    // Filter by platform compatibility
    if (!isDesktop) {
      filtered = filtered.filter(server => !server.requires_desktop)
    }

    // Sort servers
    switch (sortBy) {
      case 'popular':
        filtered.sort(
          (a, b) => (b.popularity_score || 0) - (a.popularity_score || 0),
        )
        break
      case 'name':
        filtered.sort((a, b) => a.display_name.localeCompare(b.display_name))
        break
      case 'downloads':
        filtered.sort((a, b) => (b.download_count || 0) - (a.download_count || 0))
        break
      case 'rating':
        filtered.sort((a, b) => (b.rating || 0) - (a.rating || 0))
        break
      default:
        break
    }

    return filtered
  }, [
    mcpServers,
    searchTerm,
    selectedTags,
    selectedCategories,
    selectedTransportTypes,
    sortBy,
    isDesktop,
  ])

  const filters = (
    <>
      <Select
        mode="multiple"
        placeholder="Filter by category"
        value={selectedCategories}
        onChange={setSelectedCategories}
        className="flex-1"
        allowClear
        maxTagCount="responsive"
        options={serverCategories.map(cat => ({
          key: cat,
          value: cat,
          label: cat.charAt(0).toUpperCase() + cat.slice(1),
        }))}
        popupMatchSelectWidth={false}
      />
      <Select
        mode="multiple"
        placeholder="Filter by tags"
        value={selectedTags}
        onChange={setSelectedTags}
        className="flex-1"
        allowClear
        maxTagCount="responsive"
        options={serverTags.map(tag => ({
          key: tag,
          value: tag,
          label: tag,
        }))}
        popupMatchSelectWidth={false}
      />
      <Select
        mode="multiple"
        placeholder="Transport type"
        value={selectedTransportTypes}
        onChange={setSelectedTransportTypes}
        className="flex-1"
        allowClear
        maxTagCount="responsive"
        options={transportTypes.map(type => ({
          key: type,
          value: type,
          label: type.toUpperCase(),
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
          { value: 'downloads', label: 'Downloads' },
          { value: 'rating', label: 'Rating' },
        ]}
        popupMatchSelectWidth={false}
      />
    </>
  )

  const toolbar = (
    <div className="flex gap-2 flex-wrap">
      <div className={'flex gap-2 w-full'}>
        <Input
          placeholder="Search MCP servers..."
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
  if (mcpServersLoading && !mcpServersInitialized) {
    return (
      <div className="flex justify-center items-center h-full">
        <Spin size="large" />
        <Text className="ml-4">Loading MCP servers...</Text>
      </div>
    )
  }

  // Show error state
  if (mcpServersError && !mcpServersInitialized) {
    return (
      <div className="text-center py-12">
        <Text type="danger">
          Failed to load MCP servers: {mcpServersError}
        </Text>
        <div className="mt-4">
          <Button
            onClick={() => {
              loadHubMCPServers().catch(err => {
                console.error('Failed to load hub MCP servers:', err)
                message.error('Failed to load hub MCP servers')
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
      <div className="px-3">
        {toolbar}
        {(searchTerm ||
          selectedTags.length > 0 ||
          selectedCategories.length > 0 ||
          selectedTransportTypes.length > 0) && (
          <Flex align="center" gap={8}>
            <Text type="secondary" className="text-xs">
              Filters active:{' '}
              {[
                searchTerm && 'search',
                selectedCategories.length > 0 &&
                  `${selectedCategories.length} categories`,
                selectedTags.length > 0 && `${selectedTags.length} tags`,
                selectedTransportTypes.length > 0 &&
                  `${selectedTransportTypes.length} transports`,
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

      {/* Servers List */}
      <div className="flex-1 overflow-auto px-3 pb-3">
        <div className="flex flex-col gap-3">
          {filteredServers.map(server => (
            <MCPServerCard key={server.id} server={server} />
          ))}
        </div>

        {filteredServers.length === 0 && (
          <div className="text-center py-12">
            <Text type="secondary">No MCP servers found</Text>
          </div>
        )}
      </div>
    </div>
  )
}
