import { useState, useEffect } from 'react'
import {
  Button,
  Input,
  Select,
  Typography,
  Flex,
  App,
} from 'antd'
import {
  PlusOutlined,
  SearchOutlined,
  ClearOutlined,
} from '@ant-design/icons'
import { VscFilter } from 'react-icons/vsc'
import { Stores } from '../../../../../store'
import { Permission } from '../../../../../types'
import {
  loadSystemServers,
  clearAdminMCPErrors,
} from '../../../../../store/admin/mcpServers.ts'
import { openMCPServerDrawer } from '../../../../../store/ui/mcpDrawers'
import { hasPermission } from '../../../../../permissions/utils'
import { MCPServerCard } from '../MCPServerCard'
import { useMainContentMinSize } from '../../../../hooks/useWindowMinSize'
import { MCPServerDrawer } from '../MCPServerDrawer'

const { Text } = Typography

export function SystemServersTab() {
  const { message } = App.useApp()
  const [searchTerm, setSearchTerm] = useState('')
  const [statusFilter, setStatusFilter] = useState<string>('all')
  const [sortBy, setSortBy] = useState('name')
  const [showFilters, setShowFilters] = useState(false)
  const mainContentMinSize = useMainContentMinSize()

  const {
    systemServers,
    systemServersLoading,
    systemServersError,
    systemServersInitialized,
  } = Stores.AdminMCPServers

  // Load servers on mount
  useEffect(() => {
    if (!systemServersInitialized) {
      loadSystemServers().catch(console.error)
    }
  }, [systemServersInitialized])

  // Clear error when component mounts
  useEffect(() => {
    if (systemServersError) {
      clearAdminMCPErrors()
    }
  }, [])

  const clearAllFilters = () => {
    setSearchTerm('')
    setStatusFilter('all')
  }

  const handleCreateServer = () => {
    openMCPServerDrawer(undefined, 'create-system')
  }

  // Filter and sort servers
  const filteredServers = systemServers
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
      <Button
        type="primary"
        icon={<PlusOutlined />}
        onClick={handleCreateServer}
        className="flex-1"
      >
        Add Server
      </Button>
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
  if (systemServersLoading && !systemServersInitialized) {
    return (
      <div className="flex justify-center items-center h-full">
        <Text className="ml-4">Loading system servers...</Text>
      </div>
    )
  }

  // Show error state
  if (systemServersError && !systemServersInitialized) {
    return (
      <div className="text-center py-12">
        <Text type="danger">Failed to load system servers: {systemServersError}</Text>
        <div className="mt-4">
          <Button
            onClick={() => {
              loadSystemServers().catch(err => {
                console.error('Failed to load system servers:', err)
                message.error('Failed to load system servers')
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
              isEditable={hasPermission([Permission.McpAdminServersEdit])}
            />
          ))}
        </div>

        {filteredServers.length === 0 && (
          <div className="text-center py-12">
            <Text type="secondary">
              {searchTerm || statusFilter !== 'all'
                ? 'No servers match your search criteria'
                : 'No system servers configured'}
            </Text>
          </div>
        )}
      </div>

      {/* Unified Drawer Component */}
      <MCPServerDrawer />
    </div>
  )
}
