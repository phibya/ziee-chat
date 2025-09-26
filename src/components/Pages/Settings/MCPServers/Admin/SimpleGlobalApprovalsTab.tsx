import { useState, useEffect } from 'react'
import {
  Table,
  Button,
  Input,
  Select,
  Card,
  Typography,
  Alert,
  Flex,
  Tag,
  Modal,
  Form,
  Switch,
  App,
} from 'antd'
import {
  EditOutlined,
  DeleteOutlined,
  ReloadOutlined,
  ToolOutlined,
  ClearOutlined,
} from '@ant-design/icons'
import { Stores } from '../../../../../store'
import {
  loadAllGlobalApprovals,
  setGlobalToolApproval,
  removeGlobalToolApproval,
  clearApprovalsError,
  cleanExpiredApprovals,
  clearApprovalChecksCache,
} from '../../../../../store/mcpApprovals.ts'
import type {
  ToolApprovalResponse,
  SetToolGlobalApprovalRequest,
} from '../../../../../types/api.ts'
import dayjs from 'dayjs'

const { Text } = Typography
const { Search } = Input

interface ApprovalFormData {
  approved: boolean
  auto_approve: boolean
  expires_at?: string
}

export function SimpleGlobalApprovalsTab() {
  const { message } = App.useApp()
  const [searchText, setSearchText] = useState('')
  const [serverFilter, setServerFilter] = useState<string>('all')
  const [statusFilter, setStatusFilter] = useState<string>('all')
  const [approvalModalVisible, setApprovalModalVisible] = useState(false)
  const [selectedTool, setSelectedTool] = useState<{
    serverId: string
    toolName: string
  } | null>(null)
  const [form] = Form.useForm<ApprovalFormData>()
  const [operationLoading, setOperationLoading] = useState<
    Record<string, boolean>
  >({})

  const {
    globalApprovals,
    globalApprovalsLoading,
    globalApprovalsError,
    isInitialized,
  } = Stores.MCPApprovals

  const { systemServers } = Stores.AdminMCPServers

  // Load global approvals on mount
  useEffect(() => {
    if (!isInitialized) {
      loadAllGlobalApprovals().catch(console.error)
    }
  }, [isInitialized])

  // Clear error when component mounts
  useEffect(() => {
    if (globalApprovalsError) {
      clearApprovalsError()
    }
  }, [])

  const handleEditApproval = (approval: ToolApprovalResponse) => {
    setSelectedTool({
      serverId: approval.server_id,
      toolName: approval.tool_name,
    })
    form.setFieldsValue({
      approved: approval.approved,
      auto_approve: approval.auto_approve,
      expires_at: approval.expires_at
        ? dayjs(approval.expires_at).format('YYYY-MM-DD HH:mm:ss')
        : undefined,
    })
    setApprovalModalVisible(true)
  }

  const handleDeleteApproval = async (approval: ToolApprovalResponse) => {
    try {
      setOperationLoading(prev => ({
        ...prev,
        [`delete-${approval.server_id}-${approval.tool_name}`]: true,
      }))
      await removeGlobalToolApproval(approval.server_id, approval.tool_name)
      message.success('Global approval removed successfully')
    } catch (error) {
      message.error('Failed to remove global approval')
    } finally {
      setOperationLoading(prev => ({
        ...prev,
        [`delete-${approval.server_id}-${approval.tool_name}`]: false,
      }))
    }
  }

  const handleSubmitApproval = async (values: ApprovalFormData) => {
    if (!selectedTool) return

    try {
      const request: SetToolGlobalApprovalRequest = {
        auto_approve: values.auto_approve,
        expires_at: values.expires_at,
      }

      await setGlobalToolApproval(
        selectedTool.serverId,
        selectedTool.toolName,
        request,
      )
      message.success('Global approval updated successfully')
      setApprovalModalVisible(false)
      form.resetFields()
    } catch (error) {
      message.error('Failed to update global approval')
    }
  }

  const handleCleanExpired = async () => {
    try {
      setOperationLoading(prev => ({ ...prev, 'clean-expired': true }))
      const result = await cleanExpiredApprovals()
      message.success(`Cleaned ${result.cleaned_count} expired approvals`)
    } catch (error) {
      message.error('Failed to clean expired approvals')
    } finally {
      setOperationLoading(prev => ({ ...prev, 'clean-expired': false }))
    }
  }

  const handleRefresh = async () => {
    try {
      await loadAllGlobalApprovals()
      message.success('Global approvals refreshed')
    } catch (error) {
      message.error('Failed to refresh global approvals')
    }
  }

  const handleClearCache = () => {
    clearApprovalChecksCache()
    message.success('Approval checks cache cleared')
  }

  // Convert Map to array for filtering and display
  const approvalsArray = Array.from(globalApprovals.values())

  // Filter approvals
  const filteredApprovals = approvalsArray.filter(approval => {
    const server = systemServers.find(s => s.id === approval.server_id)
    const matchesSearch =
      !searchText ||
      approval.tool_name.toLowerCase().includes(searchText.toLowerCase()) ||
      server?.display_name.toLowerCase().includes(searchText.toLowerCase())

    const matchesServer =
      serverFilter === 'all' || approval.server_id === serverFilter

    const matchesStatus =
      statusFilter === 'all' ||
      (statusFilter === 'approved' && approval.approved) ||
      (statusFilter === 'denied' && !approval.approved) ||
      (statusFilter === 'auto' && approval.auto_approve) ||
      (statusFilter === 'expired' && approval.is_expired) ||
      (statusFilter === 'active' && approval.approved && !approval.is_expired)

    return matchesSearch && matchesServer && matchesStatus
  })

  const getStatusColor = (approval: ToolApprovalResponse) => {
    if (approval.is_expired) return 'default'
    if (!approval.approved) return 'error'
    if (approval.auto_approve) return 'success'
    return 'warning'
  }

  const getStatusText = (approval: ToolApprovalResponse) => {
    if (approval.is_expired) return 'Expired'
    if (!approval.approved) return 'Denied'
    if (approval.auto_approve) return 'Auto-Approve'
    return 'Manual Approve'
  }

  const columns = [
    {
      title: 'Tool',
      key: 'tool',
      render: (approval: ToolApprovalResponse) => {
        const server = systemServers.find(s => s.id === approval.server_id)
        return (
          <div>
            <div className="flex items-center gap-2">
              <ToolOutlined className="text-xs" />
              <Text strong className="text-sm">
                {approval.tool_name}
              </Text>
            </div>
            <Text type="secondary" className="text-xs">
              {server?.display_name || approval.server_id}
            </Text>
          </div>
        )
      },
    },
    {
      title: 'Status',
      key: 'status',
      render: (approval: ToolApprovalResponse) => (
        <Tag color={getStatusColor(approval)}>{getStatusText(approval)}</Tag>
      ),
    },
    {
      title: 'Usage',
      key: 'usage',
      render: () => <Text className="text-sm">0</Text>,
    },
    {
      title: 'Expires',
      key: 'expires',
      render: (approval: ToolApprovalResponse) =>
        approval.expires_at ? (
          <Text className="text-sm">
            {dayjs(approval.expires_at).format('MMM D, YYYY')}
          </Text>
        ) : (
          <Text type="secondary">Never</Text>
        ),
    },
    {
      title: 'Actions',
      key: 'actions',
      render: (approval: ToolApprovalResponse) => (
        <div className="flex items-center gap-2">
          <Button
            type="text"
            icon={<EditOutlined />}
            onClick={() => handleEditApproval(approval)}
            size="small"
          />
          <Button
            type="text"
            danger
            icon={<DeleteOutlined />}
            onClick={() => handleDeleteApproval(approval)}
            loading={
              operationLoading[
                `delete-${approval.server_id}-${approval.tool_name}`
              ]
            }
            size="small"
          />
        </div>
      ),
    },
  ]

  return (
    <div className="space-y-4">
      {/* Statistics Cards */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <Card size="small">
          <div className="text-center">
            <div className="text-2xl font-bold text-blue-600">
              {approvalsArray.length}
            </div>
            <div className="text-sm text-gray-500">Total Approvals</div>
          </div>
        </Card>
        <Card size="small">
          <div className="text-center">
            <div className="text-2xl font-bold text-green-600">
              {approvalsArray.filter(a => a.approved && !a.is_expired).length}
            </div>
            <div className="text-sm text-gray-500">Active Approvals</div>
          </div>
        </Card>
        <Card size="small">
          <div className="text-center">
            <div className="text-2xl font-bold text-orange-600">
              {
                approvalsArray.filter(a => a.auto_approve && !a.is_expired)
                  .length
              }
            </div>
            <div className="text-sm text-gray-500">Auto-Approve</div>
          </div>
        </Card>
        <Card size="small">
          <div className="text-center">
            <div className="text-2xl font-bold text-red-600">
              {approvalsArray.filter(a => a.is_expired).length}
            </div>
            <div className="text-sm text-gray-500">Expired</div>
          </div>
        </Card>
      </div>

      {/* Filters and Actions */}
      <Card size="small">
        <Flex justify="space-between" align="center" className="mb-4">
          <div className="flex items-center gap-4 flex-wrap">
            <Search
              placeholder="Search approvals..."
              value={searchText}
              onChange={e => setSearchText(e.target.value)}
              style={{ width: 250 }}
              allowClear
            />
            <Select
              value={serverFilter}
              onChange={setServerFilter}
              style={{ width: 150 }}
            >
              <Select.Option value="all">All Servers</Select.Option>
              {systemServers.map(server => (
                <Select.Option key={server.id} value={server.id}>
                  {server.display_name}
                </Select.Option>
              ))}
            </Select>
            <Select
              value={statusFilter}
              onChange={setStatusFilter}
              style={{ width: 120 }}
            >
              <Select.Option value="all">All Status</Select.Option>
              <Select.Option value="active">Active</Select.Option>
              <Select.Option value="approved">Approved</Select.Option>
              <Select.Option value="denied">Denied</Select.Option>
              <Select.Option value="auto">Auto-Approve</Select.Option>
              <Select.Option value="expired">Expired</Select.Option>
            </Select>
          </div>
          <div className="flex items-center gap-2">
            <Button
              icon={<ClearOutlined />}
              onClick={handleClearCache}
              title="Clear Cache"
            >
              Clear Cache
            </Button>
            <Button
              onClick={handleCleanExpired}
              loading={operationLoading['clean-expired']}
              title="Clean Expired"
            >
              Clean Expired
            </Button>
            <Button
              icon={<ReloadOutlined />}
              onClick={handleRefresh}
              loading={globalApprovalsLoading}
            >
              Refresh
            </Button>
          </div>
        </Flex>

        {globalApprovalsError && (
          <Alert
            type="error"
            message="Error loading global approvals"
            description={globalApprovalsError}
            closable
            onClose={clearApprovalsError}
            className="mb-4"
          />
        )}
      </Card>

      {/* Global Approvals Table */}
      <Card>
        <Table
          columns={columns}
          dataSource={filteredApprovals}
          rowKey={approval => `${approval.server_id}-${approval.tool_name}`}
          loading={globalApprovalsLoading}
          pagination={{
            showSizeChanger: true,
            showQuickJumper: true,
            showTotal: (total, range) =>
              `${range[0]}-${range[1]} of ${total} approvals`,
          }}
        />
      </Card>

      {/* Edit Approval Modal */}
      <Modal
        title={
          selectedTool ? `${selectedTool.toolName} Approval` : 'Global Approval'
        }
        open={approvalModalVisible}
        onCancel={() => {
          setApprovalModalVisible(false)
          form.resetFields()
        }}
        footer={null}
      >
        <Form form={form} layout="vertical" onFinish={handleSubmitApproval}>
          <Form.Item name="approved" label="Approved" valuePropName="checked">
            <Switch />
          </Form.Item>

          <Form.Item
            name="auto_approve"
            label="Auto Approve"
            valuePropName="checked"
          >
            <Switch />
          </Form.Item>

          <div className="flex justify-end gap-2">
            <Button onClick={() => setApprovalModalVisible(false)}>
              Cancel
            </Button>
            <Button type="primary" htmlType="submit">
              Save Approval
            </Button>
          </div>
        </Form>
      </Modal>
    </div>
  )
}

export { SimpleGlobalApprovalsTab as GlobalApprovalsTab }
