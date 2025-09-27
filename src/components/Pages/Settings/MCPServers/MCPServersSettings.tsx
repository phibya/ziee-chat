import { useEffect, useState } from 'react'
import { App, Button, Flex, Input, Select, Spin, Typography } from 'antd'
import { PlusOutlined, SearchOutlined, ClearOutlined } from '@ant-design/icons'
import { VscFilter } from 'react-icons/vsc'
import { SettingsPageContainer } from '../common/SettingsPageContainer'
import { MCPServerCard } from './MCPServerCard'
import { MCPServerDrawer } from './MCPServerDrawer'
import { PermissionGuard } from '../../../Auth/PermissionGuard'
import { Permission } from '../../../../types'
import { Stores } from '../../../../store'
import { openMCPServerDrawer } from '../../../../store/ui/mcpDrawers'
import { clearMCPError, loadMCPServers } from '../../../../store/mcp'
import { useMainContentMinSize } from '../../../hooks/useWindowMinSize'

const { Text } = Typography

export function MCPServersSettings() {
  const { message } = App.useApp()
  const { servers, loading, error, isInitialized } = Stores.MCP
  const [searchTerm, setSearchTerm] = useState('')
  const [statusFilter, setStatusFilter] = useState('all')
  const [sortBy, setSortBy] = useState('name')
  const mainContentMinSize = useMainContentMinSize()
  const [showFilters, setShowFilters] = useState(false)

  useEffect(() => {
    if (error) {
      message.error(error)
      clearMCPError()
    }
  }, [error, message])

  const handleAddServer = () => {
    openMCPServerDrawer()
  }

  const clearAllFilters = () => {
    setSearchTerm('')
    setStatusFilter('all')
  }

  // Filter and sort servers (both user and system servers with system tag)
  const filteredServers = servers
    .filter(server => {
      const matchesSearch =
        searchTerm === '' ||
        server.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
        server.display_name.toLowerCase().includes(searchTerm.toLowerCase()) ||
        server.description?.toLowerCase().includes(searchTerm.toLowerCase())

      const matchesStatus =
        statusFilter === 'all' ||
        (statusFilter === 'active' && server.is_active) ||
        (statusFilter === 'inactive' && !server.is_active) ||
        (statusFilter === 'system' && server.is_system) ||
        (statusFilter === 'user' && !server.is_system)

      return matchesSearch && matchesStatus
    })
    .sort((a, b) => {
      switch (sortBy) {
        case 'name':
          return a.display_name.localeCompare(b.display_name)
        case 'status':
          return Number(b.is_active) - Number(a.is_active)
        case 'type':
          return Number(b.is_system) - Number(a.is_system)
        default:
          return 0
      }
    })

  const filters = (
    <>
      <Select
        placeholder="Filter by status"
        value={statusFilter}
        onChange={setStatusFilter}
        className="flex-1"
        allowClear
        options={[
          { label: 'All Servers', value: 'all' },
          { label: 'Active', value: 'active' },
          { label: 'Inactive', value: 'inactive' },
          { label: 'System', value: 'system' },
          { label: 'User', value: 'user' },
        ]}
        popupMatchSelectWidth={false}
      />
      <Select
        placeholder="Sort by"
        value={sortBy}
        onChange={setSortBy}
        className="flex-1"
        options={[
          { label: 'Name', value: 'name' },
          { label: 'Status', value: 'status' },
          { label: 'Type', value: 'type' },
        ]}
        popupMatchSelectWidth={false}
      />
      <PermissionGuard permissions={[Permission.McpServersCreate]}>
        <Button
          type="primary"
          icon={<PlusOutlined />}
          onClick={handleAddServer}
          className="flex-1"
        >
          Add Server
        </Button>
      </PermissionGuard>
    </>
  )

  const toolbar = (
    <div className="flex gap-2 flex-wrap">
      <div className={'flex gap-2 w-full'}>
        <Input
          placeholder="Search servers..."
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
  if (loading && !isInitialized) {
    return (
      <SettingsPageContainer
        title="MCP Servers"
        subtitle="Manage Model Context Protocol servers for enhanced tool capabilities"
      >
        <div className="flex justify-center items-center h-full">
          <Spin size="large" />
          <Text className="ml-4">Loading MCP servers...</Text>
        </div>
      </SettingsPageContainer>
    )
  }

  // Show error state
  if (error && !isInitialized) {
    return (
      <SettingsPageContainer
        title="MCP Servers"
        subtitle="Manage Model Context Protocol servers for enhanced tool capabilities"
      >
        <div className="text-center py-12">
          <Text type="danger">Failed to load MCP servers: {error}</Text>
          <div className="mt-4">
            <Button
              onClick={() => {
                loadMCPServers().catch((err: Error) => {
                  console.error('Failed to load MCP servers:', err)
                  message.error('Failed to load MCP servers')
                })
              }}
            >
              Retry
            </Button>
          </div>
        </div>
      </SettingsPageContainer>
    )
  }

  return (
    <SettingsPageContainer
      title="MCP Servers"
      subtitle="Manage Model Context Protocol servers for enhanced tool capabilities"
    >
      <div className="flex flex-col gap-3 h-full overflow-hidden">
        {/* Search and Filters */}
        <div className="px-0">
          {toolbar}
          {(searchTerm || statusFilter !== 'all') && (
            <Flex align="center" gap={8}>
              <Text type="secondary" className="text-xs">
                Filters active:{' '}
                {[
                  searchTerm && 'search',
                  statusFilter !== 'all' && `status: ${statusFilter}`,
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
        <div className="flex-1 overflow-auto px-0 pb-3">
          <div className="flex flex-col gap-3">
            {filteredServers.map(server => (
              <MCPServerCard
                key={server.id}
                server={server}
                isEditable={!server.is_system}
              />
            ))}
          </div>

          {filteredServers.length === 0 && (
            <div className="text-center py-12">
              <Text type="secondary">
                {searchTerm || statusFilter !== 'all'
                  ? 'No servers match your search criteria'
                  : 'No MCP servers configured'}
              </Text>
            </div>
          )}
        </div>

        {/* Drawers */}
        <MCPServerDrawer />
      </div>
    </SettingsPageContainer>
  )
}
