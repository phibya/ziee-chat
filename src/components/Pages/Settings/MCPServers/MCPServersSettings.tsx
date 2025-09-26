import { useEffect, useState } from 'react'
import { App, Button, Flex, Input, Select, Spin, Typography } from 'antd'
import { PlusOutlined, SearchOutlined, ClearOutlined } from '@ant-design/icons'
import { SettingsPageContainer } from '../common/SettingsPageContainer'
import { MCPServerCard } from './MCPServerCard'
import { AddServerDrawer } from './drawers/AddServerDrawer'
import { EditServerDrawer } from './drawers/EditServerDrawer'
import { PermissionGuard } from '../../../Auth/PermissionGuard'
import { Permission } from '../../../../types'
import { Stores } from '../../../../store'
import { openMCPServerDrawer } from '../../../../store/ui/mcpDrawers'
import { clearMCPError } from '../../../../store/mcp'

const { Text } = Typography

export function MCPServersSettings() {
  const { message } = App.useApp()

  // Use reactive proxy - automatically initializes via __init__
  const { servers, loading, error } = Stores.MCP
  const [searchTerm, setSearchTerm] = useState('')
  const [statusFilter, setStatusFilter] = useState('all')
  const [sortBy, setSortBy] = useState('name')

  // Only handle side effects like error messages
  // No need to check permissions or manually initialize - reactive proxy handles it
  useEffect(() => {
    if (error) {
      message.error(error)
      clearMCPError()
    }
  }, [error, message])

  const handleAddServer = () => {
    openMCPServerDrawer()
  }

  const clearFilters = () => {
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

  return (
    <SettingsPageContainer
      title="MCP Servers"
      subtitle="Manage Model Context Protocol servers for enhanced tool capabilities"
    >
      {/* Search and Filter Controls */}
      <Flex className="gap-3 mb-4 flex-wrap">
        <Input
          placeholder="Search servers..."
          prefix={<SearchOutlined />}
          value={searchTerm}
          onChange={e => setSearchTerm(e.target.value)}
          className="flex-1 min-w-64"
          allowClear
        />
        <Select
          placeholder="Filter by status"
          value={statusFilter}
          onChange={setStatusFilter}
          options={[
            { label: 'All Servers', value: 'all' },
            { label: 'Active', value: 'active' },
            { label: 'Inactive', value: 'inactive' },
            { label: 'System', value: 'system' },
            { label: 'User', value: 'user' },
          ]}
          className="w-40"
        />
        <Select
          placeholder="Sort by"
          value={sortBy}
          onChange={setSortBy}
          options={[
            { label: 'Name', value: 'name' },
            { label: 'Status', value: 'status' },
            { label: 'Type', value: 'type' },
          ]}
          className="w-32"
        />
        <Button
          icon={<ClearOutlined />}
          onClick={clearFilters}
          disabled={searchTerm === '' && statusFilter === 'all'}
        >
          Clear
        </Button>
        <PermissionGuard permissions={[Permission.McpServersCreate]}>
          <Button
            type="primary"
            icon={<PlusOutlined />}
            onClick={handleAddServer}
          >
            Add Server
          </Button>
        </PermissionGuard>
      </Flex>

      {/* Server Cards Grid */}
      {loading ? (
        <div className="flex justify-center p-8">
          <Spin size="large" />
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {filteredServers.map(server => (
            <MCPServerCard key={server.id} server={server} />
          ))}
          {filteredServers.length === 0 && !loading && (
            <div className="col-span-full text-center py-8">
              <Text type="secondary">
                {searchTerm || statusFilter !== 'all'
                  ? 'No servers match your search criteria'
                  : 'No MCP servers configured'}
              </Text>
            </div>
          )}
        </div>
      )}

      {/* Drawers */}
      <AddServerDrawer />
      <EditServerDrawer />
    </SettingsPageContainer>
  )
}
