import { useState, useEffect } from 'react'
import { Table, Button, Select, Card, Typography, Flex, Tag, App } from 'antd'
import {
  ReloadOutlined,
  TeamOutlined,
  DatabaseOutlined as ServerOutlined,
} from '@ant-design/icons'
import { Stores } from '../../../../../store'
import type { MCPServer } from '../../../../../types/api.ts'

const { Text } = Typography

interface GroupServerAssignment {
  groupId: string
  groupName: string
  serverIds: string[]
  servers: MCPServer[]
}

export function SimpleGroupAssignmentsTab() {
  const { message } = App.useApp()
  const [selectedGroupId, setSelectedGroupId] = useState<string>('all')

  const { systemServers } = Stores.AdminMCPServers

  // Mock user groups for now - in real implementation this would come from user groups store
  const mockGroups = [
    { id: '1', name: 'Administrators', description: 'System administrators' },
    { id: '2', name: 'Developers', description: 'Development team' },
    { id: '3', name: 'Analysts', description: 'Data analysts' },
    { id: '4', name: 'Support', description: 'Support team' },
  ]

  // Mock group assignments
  const [groupAssignments, setGroupAssignments] = useState<
    GroupServerAssignment[]
  >([])

  useEffect(() => {
    // Initialize assignments with current servers
    const assignments: GroupServerAssignment[] = [
      {
        groupId: '1',
        groupName: 'Administrators',
        serverIds: systemServers.filter(s => s.is_system).map(s => s.id),
        servers: systemServers.filter(s => s.is_system),
      },
      {
        groupId: '2',
        groupName: 'Developers',
        serverIds: systemServers
          .slice(0, Math.min(2, systemServers.length))
          .map(s => s.id),
        servers: systemServers.slice(0, Math.min(2, systemServers.length)),
      },
      {
        groupId: '3',
        groupName: 'Analysts',
        serverIds: systemServers
          .filter(s => s.tool_count && s.tool_count > 0)
          .map(s => s.id),
        servers: systemServers.filter(s => s.tool_count && s.tool_count > 0),
      },
      {
        groupId: '4',
        groupName: 'Support',
        serverIds: [],
        servers: [],
      },
    ]
    setGroupAssignments(assignments)
  }, [systemServers])

  const handleRefreshAssignments = async () => {
    try {
      // In real implementation: await loadGroupServerAssignments()
      message.success('Group assignments refreshed')
    } catch (error) {
      message.error('Failed to refresh group assignments')
    }
  }

  // Filter assignments based on selected group
  const filteredAssignments =
    selectedGroupId === 'all'
      ? groupAssignments
      : groupAssignments.filter(a => a.groupId === selectedGroupId)

  const columns = [
    {
      title: 'Group',
      key: 'group',
      render: (assignment: GroupServerAssignment) => {
        const group = mockGroups.find(g => g.id === assignment.groupId)
        return (
          <div>
            <div className="flex items-center gap-2">
              <TeamOutlined />
              <Text strong>{assignment.groupName}</Text>
            </div>
            {group?.description && (
              <Text type="secondary" className="text-sm block">
                {group.description}
              </Text>
            )}
          </div>
        )
      },
    },
    {
      title: 'Assigned Servers',
      key: 'servers',
      render: (assignment: GroupServerAssignment) => (
        <div className="space-y-1">
          {assignment.servers.length === 0 ? (
            <Text type="secondary">No servers assigned</Text>
          ) : (
            <div className="flex flex-wrap gap-1">
              {assignment.servers.map(server => (
                <Tag
                  key={server.id}
                  color={server.is_active ? 'green' : 'default'}
                >
                  <ServerOutlined className="mr-1" />
                  {server.display_name}
                </Tag>
              ))}
            </div>
          )}
          <Text type="secondary" className="text-xs">
            {assignment.servers.length} server
            {assignment.servers.length !== 1 ? 's' : ''} assigned
          </Text>
        </div>
      ),
    },
    {
      title: 'Active Tools',
      key: 'tools',
      render: (assignment: GroupServerAssignment) => {
        const totalTools = assignment.servers.reduce(
          (sum, server) => sum + (server.tool_count || 0),
          0,
        )
        const activeTools = assignment.servers
          .filter(s => s.is_active)
          .reduce((sum, server) => sum + (server.tool_count || 0), 0)
        return (
          <div>
            <Text className="text-sm">
              {activeTools} / {totalTools}
            </Text>
            <Text type="secondary" className="text-xs block">
              Active / Total Tools
            </Text>
          </div>
        )
      },
    },
  ]

  return (
    <div className="space-y-4">
      {/* Statistics Cards */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <Card size="small">
          <div className="text-center">
            <div className="text-2xl font-bold text-blue-600">
              {mockGroups.length}
            </div>
            <div className="text-sm text-gray-500">Total Groups</div>
          </div>
        </Card>
        <Card size="small">
          <div className="text-center">
            <div className="text-2xl font-bold text-green-600">
              {groupAssignments.reduce((sum, a) => sum + a.servers.length, 0)}
            </div>
            <div className="text-sm text-gray-500">Server Assignments</div>
          </div>
        </Card>
        <Card size="small">
          <div className="text-center">
            <div className="text-2xl font-bold text-orange-600">
              {groupAssignments.reduce(
                (sum, a) => sum + a.servers.filter(s => s.is_active).length,
                0,
              )}
            </div>
            <div className="text-sm text-gray-500">Active Servers</div>
          </div>
        </Card>
        <Card size="small">
          <div className="text-center">
            <div className="text-2xl font-bold text-purple-600">
              {groupAssignments.reduce(
                (sum, a) =>
                  sum +
                  a.servers.reduce(
                    (toolSum, s) => toolSum + (s.tool_count || 0),
                    0,
                  ),
                0,
              )}
            </div>
            <div className="text-sm text-gray-500">Available Tools</div>
          </div>
        </Card>
      </div>

      {/* Filters and Actions */}
      <Card size="small">
        <Flex justify="space-between" align="center" className="mb-4">
          <div className="flex items-center gap-4">
            <Select
              value={selectedGroupId}
              onChange={setSelectedGroupId}
              style={{ width: 200 }}
            >
              <Select.Option value="all">All Groups</Select.Option>
              {mockGroups.map(group => (
                <Select.Option key={group.id} value={group.id}>
                  {group.name}
                </Select.Option>
              ))}
            </Select>
          </div>
          <Button icon={<ReloadOutlined />} onClick={handleRefreshAssignments}>
            Refresh
          </Button>
        </Flex>
      </Card>

      {/* Group Assignments Table */}
      <Card>
        <Table
          columns={columns}
          dataSource={filteredAssignments}
          rowKey="groupId"
          pagination={{
            showSizeChanger: true,
            showQuickJumper: true,
            showTotal: (total, range) =>
              `${range[0]}-${range[1]} of ${total} groups`,
          }}
        />
      </Card>
    </div>
  )
}

export { SimpleGroupAssignmentsTab as GroupAssignmentsTab }
