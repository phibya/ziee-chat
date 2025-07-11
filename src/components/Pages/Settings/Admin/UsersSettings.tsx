import { useEffect, useState } from 'react'
import {
  App,
  Badge,
  Button,
  Card,
  Drawer,
  Flex,
  Form,
  Input,
  List,
  Modal,
  Popconfirm,
  Result,
  Select,
  Space,
  Switch,
  Table,
  Tag,
  Typography,
} from 'antd'
import {
  EditOutlined,
  ExclamationCircleOutlined,
  LockOutlined,
  PlusOutlined,
  TeamOutlined,
  UserOutlined,
} from '@ant-design/icons'
import type { ColumnsType } from 'antd/es/table'
import { useShallow } from 'zustand/react/shallow'
import { isDesktopApp } from '../../../../api/core.ts'
import {
  ResetPasswordRequest,
  UpdateUserRequest,
  User,
  UserGroup,
} from '../../../../types'
import { Permission, usePermissions } from '../../../../permissions'
import { useAdminStore } from '../../../../store/admin'
import { PageContainer } from '../../../common/PageContainer'
import { UserRegistrationSettings } from './UserRegistrationSettings.tsx'

const { Title, Text } = Typography
const { Option } = Select

export function UsersSettings() {
  const { message } = App.useApp()
  const { hasPermission } = usePermissions()

  // Admin store
  const {
    users,
    groups,
    loading,
    error,
    loadUsers,
    loadGroups,
    updateUser,
    resetUserPassword,
    toggleUserActive,
    assignUserToGroup,
    removeUserFromGroup,
    clearError,
  } = useAdminStore(
    useShallow(state => ({
      users: state.users,
      groups: state.groups,
      loading: state.loading,
      updating: state.updating,
      error: state.error,
      loadUsers: state.loadUsers,
      loadGroups: state.loadGroups,
      updateUser: state.updateUser,
      resetUserPassword: state.resetUserPassword,
      toggleUserActive: state.toggleUserActive,
      assignUserToGroup: state.assignUserToGroup,
      removeUserFromGroup: state.removeUserFromGroup,
      clearError: state.clearError,
    })),
  )

  const [editModalVisible, setEditModalVisible] = useState(false)
  const [passwordModalVisible, setPasswordModalVisible] = useState(false)
  const [groupsDrawerVisible, setGroupsDrawerVisible] = useState(false)
  const [assignGroupModalVisible, setAssignGroupModalVisible] = useState(false)
  const [selectedUser, setSelectedUser] = useState<User | null>(null)
  const [editForm] = Form.useForm()
  const [passwordForm] = Form.useForm()
  const [assignGroupForm] = Form.useForm()

  // Check permissions
  const canReadUsers = hasPermission(Permission.users.read)
  const canEditUsers = hasPermission(Permission.users.edit)

  // User needs at least read permission to access this page
  const canAccessUsers = canReadUsers

  // Redirect if desktop app or insufficient permissions
  useEffect(() => {
    if (isDesktopApp) {
      message.warning('User management is not available in desktop mode')
      return
    }
    if (!canAccessUsers) {
      message.warning('You do not have permission to access user management')
      return
    }
    loadUsers()
    loadGroups()
  }, [canAccessUsers, loadUsers, loadGroups])

  // Show errors
  useEffect(() => {
    if (error) {
      message.error(error)
      clearError()
    }
  }, [error, message, clearError])

  const handleEditUser = async (values: any) => {
    if (!selectedUser) return

    try {
      const updateData: UpdateUserRequest = {
        user_id: selectedUser.id,
        username: values.username,
        email: values.email,
        is_active: values.is_active,
        profile: values.profile ? JSON.parse(values.profile) : undefined,
      }

      await updateUser(selectedUser.id, updateData)

      message.success('User updated successfully')
      setEditModalVisible(false)
      setSelectedUser(null)
      editForm.resetFields()
    } catch (error) {
      console.error('Failed to update user:', error)
      // Error is handled by the store
    }
  }

  const handleResetPassword = async (values: any) => {
    if (!selectedUser) return

    try {
      const resetData: ResetPasswordRequest = {
        user_id: selectedUser.id,
        new_password: values.new_password,
      }

      await resetUserPassword(selectedUser.id, resetData.new_password)

      message.success('Password reset successfully')
      setPasswordModalVisible(false)
      setSelectedUser(null)
      passwordForm.resetFields()
    } catch (error) {
      console.error('Failed to reset password:', error)
      // Error is handled by the store
    }
  }

  const handleToggleActive = async (userId: string) => {
    try {
      await toggleUserActive(userId)
      message.success('User status updated successfully')
    } catch (error) {
      console.error('Failed to update user status:', error)
      // Error is handled by the store
    }
  }

  const handleAssignGroup = async (values: any) => {
    if (!selectedUser) return

    try {
      await assignUserToGroup(selectedUser.id, values.group_id)
      message.success('User assigned to group successfully')
      setAssignGroupModalVisible(false)
      setSelectedUser(null)
      assignGroupForm.resetFields()
    } catch (error) {
      console.error('Failed to assign user to group:', error)
      // Error is handled by the store
    }
  }

  const handleRemoveFromGroup = async (userId: string, groupId: string) => {
    try {
      await removeUserFromGroup(userId, groupId)

      message.success('User removed from group successfully')
    } catch (error) {
      console.error('Failed to remove user from group:', error)
      // Error is handled by the store
    }
  }

  const openEditModal = (user: User) => {
    setSelectedUser(user)
    editForm.setFieldsValue({
      username: user.username,
      email: user.emails[0]?.address,
      is_active: user.is_active,
      profile: user.profile ? JSON.stringify(user.profile, null, 2) : '',
    })
    setEditModalVisible(true)
  }

  const openPasswordModal = (user: User) => {
    setSelectedUser(user)
    setPasswordModalVisible(true)
  }

  const openGroupsDrawer = (user: User) => {
    setSelectedUser(user)
    setGroupsDrawerVisible(true)
  }

  const openAssignGroupModal = (user: User) => {
    setSelectedUser(user)
    setAssignGroupModalVisible(true)
  }

  const columns: ColumnsType<User> = [
    {
      title: 'User',
      key: 'user',
      render: (_, record: User) => (
        <Space>
          <UserOutlined />
          <div>
            <div>
              <span>{record.username}</span>
              {record.is_protected && (
                <Tag color="gold" className="ml-2">
                  Protected
                </Tag>
              )}
            </div>
            <Text type="secondary" className="text-xs">
              {record.emails[0]?.address}
            </Text>
          </div>
        </Space>
      ),
    },
    {
      title: 'Status',
      dataIndex: 'is_active',
      key: 'is_active',
      render: (active: boolean) => (
        <Badge
          status={active ? 'success' : 'error'}
          text={active ? 'Active' : 'Inactive'}
        />
      ),
    },
    {
      title: 'Groups',
      dataIndex: 'groups',
      key: 'groups',
      render: (groups: UserGroup[]) => (
        <div>
          {groups.slice(0, 2).map(group => (
            <Tag key={group.id} color="blue">
              {group.name}
            </Tag>
          ))}
          {groups.length > 2 && <Tag>+{groups.length - 2} more</Tag>}
        </div>
      ),
    },
    {
      title: 'Last Login',
      dataIndex: 'last_login_at',
      key: 'last_login_at',
      render: (date: string) =>
        date ? (
          new Date(date).toLocaleDateString()
        ) : (
          <Text type="secondary">Never</Text>
        ),
    },
    {
      title: 'Created',
      dataIndex: 'created_at',
      key: 'created_at',
      render: (date: string) => new Date(date).toLocaleDateString(),
    },
    {
      title: 'Actions',
      key: 'actions',
      render: (_, record: User) => (
        <Space>
          {canEditUsers && !record.is_protected && (
            <Button
              type="link"
              icon={<EditOutlined />}
              onClick={() => openEditModal(record)}
            >
              Edit
            </Button>
          )}
          {canEditUsers && !record.is_protected && (
            <Button
              type="link"
              icon={<LockOutlined />}
              onClick={() => openPasswordModal(record)}
            >
              Reset Password
            </Button>
          )}
          {canReadUsers && (
            <Button
              type="link"
              icon={<TeamOutlined />}
              onClick={() => openGroupsDrawer(record)}
            >
              Groups
            </Button>
          )}
          {canEditUsers && !(record.is_protected && record.is_active) && (
            <Popconfirm
              title={`${record.is_active ? 'Deactivate' : 'Activate'} this user?`}
              onConfirm={() => handleToggleActive(record.id)}
              okText="Yes"
              cancelText="No"
            >
              <Button type="link" danger={record.is_active}>
                {record.is_active ? 'Deactivate' : 'Activate'}
              </Button>
            </Popconfirm>
          )}
        </Space>
      ),
    },
  ]

  if (isDesktopApp) {
    return (
      <Card>
        <div className="text-center">
          <Title level={4}>User Management</Title>
          <Text type="secondary">
            User management is disabled in desktop mode
          </Text>
        </div>
      </Card>
    )
  }

  if (!canAccessUsers) {
    return (
      <Result
        icon={<ExclamationCircleOutlined />}
        title="Access Denied"
        subTitle={`You do not have permission to access user management. Contact your administrator to request ${Permission.users.read} permission.`}
        extra={
          <Button type="primary" onClick={() => window.history.back()}>
            Go Back
          </Button>
        }
      />
    )
  }

  return (
    <PageContainer>
      <div>
        <div className="flex justify-between items-center mb-6">
          <Title level={3}>Users</Title>
        </div>

        {/* User Registration Settings */}
        <Flex vertical className="gap-6">
          <UserRegistrationSettings />

          <Card>
            <Table
              columns={columns}
              dataSource={users}
              rowKey="id"
              loading={loading}
              pagination={{
                pageSize: 10,
                showSizeChanger: true,
                showTotal: total => `Total ${total} users`,
              }}
            />
          </Card>
        </Flex>

        {/* Edit User Modal */}
        <Modal
          title="Edit User"
          open={editModalVisible}
          onCancel={() => {
            setEditModalVisible(false)
            setSelectedUser(null)
            editForm.resetFields()
          }}
          footer={null}
          width={600}
          maskClosable={false}
        >
          <Form form={editForm} layout="vertical" onFinish={handleEditUser}>
            <Form.Item
              name="username"
              label="Username"
              rules={[{ required: true, message: 'Please enter username' }]}
            >
              <Input placeholder="Enter username" />
            </Form.Item>
            <Form.Item
              name="email"
              label="Email"
              rules={[
                {
                  required: true,
                  type: 'email',
                  message: 'Please enter valid email',
                },
              ]}
            >
              <Input placeholder="Enter email" />
            </Form.Item>
            <Form.Item name="is_active" label="Active" valuePropName="checked">
              <Switch />
            </Form.Item>
            <Form.Item
              name="profile"
              label="Profile (JSON)"
              rules={[
                {
                  validator: (_, value) => {
                    if (!value) return Promise.resolve()
                    try {
                      JSON.parse(value)
                      return Promise.resolve()
                    } catch {
                      return Promise.reject('Invalid JSON format')
                    }
                  },
                },
              ]}
            >
              <Input.TextArea rows={4} placeholder='{"name": "John Doe"}' />
            </Form.Item>
            <Form.Item className="mb-0">
              <Space>
                <Button type="primary" htmlType="submit">
                  Update User
                </Button>
                <Button
                  onClick={() => {
                    setEditModalVisible(false)
                    setSelectedUser(null)
                    editForm.resetFields()
                  }}
                >
                  Cancel
                </Button>
              </Space>
            </Form.Item>
          </Form>
        </Modal>

        {/* Reset Password Modal */}
        <Modal
          title="Reset Password"
          open={passwordModalVisible}
          onCancel={() => {
            setPasswordModalVisible(false)
            setSelectedUser(null)
            passwordForm.resetFields()
          }}
          footer={null}
          maskClosable={false}
        >
          <Form
            form={passwordForm}
            layout="vertical"
            onFinish={handleResetPassword}
          >
            <Form.Item
              name="new_password"
              label="New Password"
              rules={[
                { required: true, message: 'Please enter new password' },
                { min: 6, message: 'Password must be at least 6 characters' },
              ]}
            >
              <Input.Password placeholder="Enter new password" />
            </Form.Item>
            <Form.Item
              name="confirm_password"
              label="Confirm Password"
              dependencies={['new_password']}
              rules={[
                { required: true, message: 'Please confirm password' },
                ({ getFieldValue }) => ({
                  validator(_, value) {
                    if (!value || getFieldValue('new_password') === value) {
                      return Promise.resolve()
                    }
                    return Promise.reject('Passwords do not match')
                  },
                }),
              ]}
            >
              <Input.Password placeholder="Confirm new password" />
            </Form.Item>
            <Form.Item className="mb-0">
              <Space>
                <Button type="primary" htmlType="submit">
                  Reset Password
                </Button>
                <Button
                  onClick={() => {
                    setPasswordModalVisible(false)
                    setSelectedUser(null)
                    passwordForm.resetFields()
                  }}
                >
                  Cancel
                </Button>
              </Space>
            </Form.Item>
          </Form>
        </Modal>

        {/* Groups Drawer */}
        <Drawer
          title={`Groups for ${selectedUser?.username}`}
          placement="right"
          onClose={() => setGroupsDrawerVisible(false)}
          open={groupsDrawerVisible}
          width={400}
          extra={
            canEditUsers &&
            !selectedUser?.is_protected && (
              <Button
                type="primary"
                icon={<PlusOutlined />}
                onClick={() => {
                  setGroupsDrawerVisible(false)
                  openAssignGroupModal(selectedUser!)
                }}
              >
                Assign Group
              </Button>
            )
          }
        >
          <List
            dataSource={selectedUser?.groups || []}
            renderItem={group => (
              <List.Item
                actions={[
                  canEditUsers && !selectedUser?.is_protected && (
                    <Popconfirm
                      key="remove"
                      title="Remove user from this group?"
                      onConfirm={() =>
                        handleRemoveFromGroup(selectedUser!.id, group.id)
                      }
                      okText="Yes"
                      cancelText="No"
                    >
                      <Button type="link" danger size="small">
                        Remove
                      </Button>
                    </Popconfirm>
                  ),
                ].filter(Boolean)}
              >
                <List.Item.Meta
                  avatar={<TeamOutlined />}
                  title={group.name}
                  description={group.description}
                />
              </List.Item>
            )}
          />
        </Drawer>

        {/* Assign Group Modal */}
        <Modal
          title="Assign User to Group"
          open={assignGroupModalVisible}
          onCancel={() => {
            setAssignGroupModalVisible(false)
            setSelectedUser(null)
            assignGroupForm.resetFields()
          }}
          footer={null}
          maskClosable={false}
        >
          <Form
            form={assignGroupForm}
            layout="vertical"
            onFinish={handleAssignGroup}
          >
            <Form.Item
              name="group_id"
              label="Select Group"
              rules={[{ required: true, message: 'Please select a group' }]}
            >
              <Select placeholder="Select a group to assign">
                {groups
                  .filter(
                    group =>
                      !selectedUser?.groups.some(ug => ug.id === group.id),
                  )
                  .map(group => (
                    <Option key={group.id} value={group.id}>
                      {group.name}
                    </Option>
                  ))}
              </Select>
            </Form.Item>
            <Form.Item className="mb-0">
              <Space>
                <Button type="primary" htmlType="submit">
                  Assign Group
                </Button>
                <Button
                  onClick={() => {
                    setAssignGroupModalVisible(false)
                    setSelectedUser(null)
                    assignGroupForm.resetFields()
                  }}
                >
                  Cancel
                </Button>
              </Space>
            </Form.Item>
          </Form>
        </Modal>
      </div>
    </PageContainer>
  )
}
