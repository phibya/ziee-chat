import {
  EditOutlined,
  ExclamationCircleOutlined,
  LockOutlined,
  PlusOutlined,
  TeamOutlined,
  UserOutlined,
} from '@ant-design/icons'
import {
  App,
  Badge,
  Button,
  Card,
  Flex,
  Form,
  Input,
  List,
  Popconfirm,
  Result,
  Select,
  Switch,
  Table,
  Tag,
  Typography,
} from 'antd'
import { Drawer } from '../../../common/Drawer.tsx'
import type { ColumnsType } from 'antd/es/table'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { isDesktopApp } from '../../../../api/core.ts'
import { Permission, usePermissions } from '../../../../permissions'
import {
  assignUserToUserGroup,
  clearSystemAdminError,
  loadAllSystemUsers,
  loadAllUserGroups,
  removeUserFromUserGroup,
  resetSystemUserPassword,
  Stores,
  toggleSystemUserActiveStatus,
  updateSystemUser,
} from '../../../../store'
import {
  ResetPasswordRequest,
  UpdateUserRequest,
  User,
  UserGroup,
} from '../../../../types'
import { PageContainer } from '../../../common/PageContainer'
import { UserRegistrationSettings } from './UserRegistrationSettings.tsx'

const { Title, Text } = Typography
const { Option } = Select

export function UsersSettings() {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const { hasPermission } = usePermissions()

  // Admin store
  const { users, groups, loading, error } = Stores.Admin

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
    loadAllSystemUsers()
    loadAllUserGroups()
  }, [canAccessUsers])

  // Show errors
  useEffect(() => {
    if (error) {
      message.error(error)
      clearSystemAdminError()
    }
  }, [error, message])

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

      await updateSystemUser(selectedUser.id, updateData)

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

      await resetSystemUserPassword(selectedUser.id, resetData.new_password)

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
      await toggleSystemUserActiveStatus(userId)
      message.success('User status updated successfully')
    } catch (error) {
      console.error('Failed to update user status:', error)
      // Error is handled by the store
    }
  }

  const handleAssignGroup = async (values: any) => {
    if (!selectedUser) return

    try {
      await assignUserToUserGroup(selectedUser.id, values.group_id)
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
      await removeUserFromUserGroup(userId, groupId)

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
      title: t('admin.users.columns.user'),
      key: 'user',
      render: (_, record: User) => (
        <Flex className="gap-2">
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
        </Flex>
      ),
    },
    {
      title: t('admin.users.columns.status'),
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
      title: t('admin.users.columns.groups'),
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
      title: t('admin.users.columns.lastLogin'),
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
      title: t('admin.users.columns.created'),
      dataIndex: 'created_at',
      key: 'created_at',
      render: (date: string) => new Date(date).toLocaleDateString(),
    },
    {
      title: t('admin.users.columns.actions'),
      key: 'actions',
      render: (_, record: User) => (
        <Flex className="gap-2">
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
        </Flex>
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
        title={t('admin.users.accessDenied')}
        subTitle={t('admin.users.accessDeniedMessage', {
          permission: Permission.users.read,
        })}
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
        <Flex vertical className="gap-3">
          <UserRegistrationSettings />

          <Card
            styles={{
              body: {
                padding: '0',
              },
            }}
          >
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
        <Drawer
          title={t('admin.users.editUser')}
          open={editModalVisible}
          onClose={() => {
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
              label={t('admin.users.forms.username')}
              rules={[{ required: true, message: 'Please enter username' }]}
            >
              <Input placeholder={t('admin.users.forms.enterUsername')} />
            </Form.Item>
            <Form.Item
              name="email"
              label={t('admin.users.forms.email')}
              rules={[
                {
                  required: true,
                  type: 'email',
                  message: 'Please enter valid email',
                },
              ]}
            >
              <Input placeholder={t('admin.users.forms.enterEmail')} />
            </Form.Item>
            <Form.Item
              name="is_active"
              label={t('admin.users.forms.active')}
              valuePropName="checked"
            >
              <Switch />
            </Form.Item>
            <Form.Item
              name="profile"
              label={t('admin.users.forms.profile')}
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
              <Flex className="gap-2">
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
              </Flex>
            </Form.Item>
          </Form>
        </Drawer>

        {/* Reset Password Modal */}
        <Drawer
          title={t('admin.users.resetPassword')}
          open={passwordModalVisible}
          onClose={() => {
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
              label={t('admin.users.forms.newPassword')}
              rules={[
                { required: true, message: 'Please enter new password' },
                { min: 6, message: 'Password must be at least 6 characters' },
              ]}
            >
              <Input.Password
                placeholder={t('admin.users.forms.enterNewPassword')}
              />
            </Form.Item>
            <Form.Item
              name="confirm_password"
              label={t('admin.users.forms.confirmPassword')}
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
              <Input.Password
                placeholder={t('admin.users.forms.confirmNewPassword')}
              />
            </Form.Item>
            <Form.Item className="mb-0">
              <Flex className="gap-2">
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
              </Flex>
            </Form.Item>
          </Form>
        </Drawer>

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
                      title={t('admin.users.removeFromGroupConfirm')}
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
        <Drawer
          title={t('admin.users.assignToGroup')}
          open={assignGroupModalVisible}
          onClose={() => {
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
              label={t('admin.users.forms.selectGroup')}
              rules={[
                {
                  required: true,
                  message: t('admin.users.forms.pleaseSelectGroup'),
                },
              ]}
            >
              <Select placeholder={t('admin.users.forms.selectGroupToAssign')}>
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
              <Flex className="gap-2">
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
              </Flex>
            </Form.Item>
          </Form>
        </Drawer>
      </div>
    </PageContainer>
  )
}
