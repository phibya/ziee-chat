import { ClearOutlined, SearchOutlined } from '@ant-design/icons'
import { Button, Flex, Input, Select, Typography } from 'antd'
import { useEffect, useMemo, useState } from 'react'
import { searchModels, useHubStore } from '../../../store/hub'
import { ModelCard } from './ModelCard'
import {
  loadAllModelProviders,
  loadAllAdminModelRepositories,
} from '../../../store'
import { useMainContentMinSize } from '../../hooks/useWindowMinSize.ts'
import { VscFilter } from 'react-icons/vsc'

const { Text } = Typography

export function ModelsTab() {
  const { models } = useHubStore()
  const [searchTerm, setSearchTerm] = useState('')
  const [selectedTags, setSelectedTags] = useState<string[]>([])
  const [selectedCapabilities, setSelectedCapabilities] = useState<string[]>([])
  const [sortBy, setSortBy] = useState('popular')
  const mainContentMinSize = useMainContentMinSize()
  const [showFilters, setShowFilters] = useState(false)

  useEffect(() => {
    loadAllAdminModelRepositories()
    loadAllModelProviders()
  }, [])

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
    if (hasCode)
      capabilities.push({ key: 'code_interpreter', label: 'Code Interpreter' })
    if (hasAudio) capabilities.push({ key: 'audio', label: 'Audio' })

    return capabilities
  }, [models])

  const filteredModels = useMemo(() => {
    let filtered = searchModels(models, searchTerm)

    // Filter by tags
    if (selectedTags.length > 0) {
      filtered = filtered.filter(model =>
        selectedTags.some(tag => model.tags.includes(tag)),
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
        options={modelTags.map(tag => ({
          key: tag,
          value: tag,
          label: tag,
        }))}
        popupMatchSelectWidth={false}
      />
      <Select
        mode="multiple"
        placeholder="Capabilities"
        value={selectedCapabilities}
        onChange={setSelectedCapabilities}
        className="flex-1"
        allowClear
        maxTagCount="responsive"
        options={modelCapabilities.map(capability => ({
          key: capability.key,
          value: capability.key,
          label: capability.label,
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
          { value: 'size', label: 'Size' },
        ]}
        popupMatchSelectWidth={false}
      />
    </>
  )

  const toolbar = (
    <div className="flex gap-2 flex-wrap">
      <div className={'flex gap-2 w-full'}>
        <Input
          placeholder="Search models..."
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

  return (
    <div className="flex flex-col gap-3 h-full overflow-hidden">
      {/* Search and Filters */}
      <div className="px-3">
        {toolbar}
        {(searchTerm ||
          selectedTags.length > 0 ||
          selectedCapabilities.length > 0) && (
          <Flex align="center" gap={8}>
            <Text type="secondary" className="text-xs">
              Filters active:{' '}
              {[
                searchTerm && 'search',
                selectedTags.length > 0 && `${selectedTags.length} tags`,
                selectedCapabilities.length > 0 &&
                  `${selectedCapabilities.length} capabilities`,
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

      {/* Models List */}
      <div className="flex-1 overflow-auto px-3 pb-3">
        <div className="flex flex-col gap-3">
          {filteredModels.map(model => (
            <ModelCard key={model.id} model={model} />
          ))}
        </div>

        {filteredModels.length === 0 && (
          <div className="text-center py-12">
            <Text type="secondary">No models found</Text>
          </div>
        )}
      </div>
    </div>
  )
}
