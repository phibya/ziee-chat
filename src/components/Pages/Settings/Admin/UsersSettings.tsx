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
  Descriptions,
  Divider,
  Empty,
  Flex,
  Form,
  Input,
  List,
  Pagination,
  Popconfirm,
  Result,
  Select,
  Spin,
  Switch,
  Tag,
  Typography,
} from 'antd'
import { Drawer } from '../../../Common/Drawer'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { isTauriView } from '../../../../api/core.ts'
import { Permission, usePermissions } from '../../../../permissions'
import {
  assignUserToUserGroup,
  clearAdminUserGroupsStoreError,
  clearAdminUsersStoreError,
  loadSystemUsers,
  loadUserGroups,
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
} from '../../../../types'
import { SettingsPageContainer } from '../common/SettingsPageContainer.tsx'
import { UserRegistrationSettings } from './UserRegistrationSettings.tsx'

const { Title, Text } = Typography
const { Option } = Select

export function UsersSettings() {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const { hasPermission } = usePermissions()

  // Admin stores
  const {
    users,
    total: totalUsers,
    currentPage: storePage,
    pageSize: storePageSize,
    loading: loadingUsers,
    error: usersError,
  } = Stores.AdminUsers
  const { groups, error: groupsError } = Stores.AdminUserGroups

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
    if (isTauriView) {
      message.warning('User management is not available in desktop mode')
      return
    }
    if (!canAccessUsers) {
      message.warning('You do not have permission to access user management')
      return
    }
    loadSystemUsers(1, 10)
    loadUserGroups()
  }, [canAccessUsers])

  // Show errors
  useEffect(() => {
    if (usersError) {
      message.error(usersError)
      clearAdminUsersStoreError()
    }
    if (groupsError) {
      message.error(groupsError)
      clearAdminUserGroupsStoreError()
    }
  }, [usersError, groupsError, message])

  const handleEditUser = async (values: any) => {
    if (!selectedUser) return

    try {
      const updateData: { user_id: string } & UpdateUserRequest = {
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

  const getUserActions = (user: User) => {
    const actions: React.ReactNode[] = []

    // Always include the active/inactive status switch first
    if (canEditUsers && !(user.is_protected && user.is_active)) {
      actions.push(
        <Popconfirm
          key="active-confirm"
          title={`${user.is_active ? 'Deactivate' : 'Activate'} this user?`}
          onConfirm={() => handleToggleActive(user.id)}
          okText="Yes"
          cancelText="No"
        >
          <Switch
            className={'!mr-2'}
            checked={user.is_active}
            disabled={!canEditUsers}
          />
        </Popconfirm>,
      )
    }

    if (canEditUsers && !user.is_protected) {
      actions.push(
        <Button
          key="edit"
          type="text"
          icon={<EditOutlined />}
          onClick={() => openEditModal(user)}
        >
          Edit
        </Button>,
      )

      actions.push(
        <Button
          key="password"
          type="text"
          icon={<LockOutlined />}
          onClick={() => openPasswordModal(user)}
        >
          Reset Password
        </Button>,
      )
    }

    if (canReadUsers) {
      actions.push(
        <Button
          key="groups"
          type="text"
          icon={<TeamOutlined />}
          onClick={() => openGroupsDrawer(user)}
        >
          Groups
        </Button>,
      )
    }

    return actions.filter(Boolean)
  }

  const handlePageChange = (page: number, size?: number) => {
    const newPageSize = size || storePageSize
    const newPage = size && size !== storePageSize ? 1 : page // Reset to page 1 if page size changes

    loadSystemUsers(newPage, newPageSize)
  }

  if (isTauriView) {
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
    <SettingsPageContainer title="Users">
      <div>
        {/* User Registration Settings */}
        <Flex vertical className="gap-3">
          <UserRegistrationSettings />

          <Card title={t('admin.users.title')}>
            {loadingUsers ? (
              <div className="flex justify-center py-8">
                <Spin size="large" />
              </div>
            ) : users.length === 0 ? (
              <div>
                <Empty description="No users found" />
              </div>
            ) : (
              <div>
                {users.map((user, index) => (
                  <div key={user.id}>
                    <div className="flex items-start gap-3 flex-wrap">
                      {/* User Info */}
                      <div className="flex-1">
                        <div className="flex items-center gap-2 mb-2 flex-wrap">
                          <div className={'flex-1 min-w-48'}>
                            <Flex className="gap-2 items-center">
                              <UserOutlined />
                              <Text className="font-medium">
                                {user.username}
                              </Text>
                              {user.is_protected && (
                                <Tag color="gold">Protected</Tag>
                              )}
                              <Badge
                                status={user.is_active ? 'success' : 'error'}
                                text={user.is_active ? 'Active' : 'Inactive'}
                              />
                            </Flex>
                          </div>
                          <div
                            className={'flex gap-1 items-center justify-end'}
                          >
                            {getUserActions(user)}
                          </div>
                        </div>

                        <Descriptions
                          size="small"
                          column={{ xs: 1, sm: 2, md: 3 }}
                          colon={false}
                          labelStyle={{ fontSize: '12px', color: '#8c8c8c' }}
                          contentStyle={{ fontSize: '12px' }}
                        >
                          <Descriptions.Item label="Email">
                            {user.emails[0]?.address}
                          </Descriptions.Item>
                          <Descriptions.Item label="Last Login">
                            {user.last_login_at
                              ? new Date(
                                  user.last_login_at,
                                ).toLocaleDateString()
                              : 'Never'}
                          </Descriptions.Item>
                          <Descriptions.Item label="Created">
                            {new Date(user.created_at).toLocaleDateString()}
                          </Descriptions.Item>
                          {user.groups && user.groups.length > 0 && (
                            <Descriptions.Item
                              label="Groups"
                              span={{ xs: 1, sm: 2, md: 3 }}
                            >
                              <Flex wrap className="gap-1">
                                {user.groups.slice(0, 3).map(group => (
                                  <Tag
                                    key={group.id}
                                    color="blue"
                                    className="text-xs"
                                  >
                                    {group.name}
                                  </Tag>
                                ))}
                                {user.groups.length > 3 && (
                                  <Tag className="text-xs">
                                    +{user.groups.length - 3} more
                                  </Tag>
                                )}
                              </Flex>
                            </Descriptions.Item>
                          )}
                        </Descriptions>
                      </div>
                    </div>
                    {index < users.length - 1 && <Divider className="my-0" />}
                  </div>
                ))}
              </div>
            )}

            {users.length > 0 && (
              <>
                <Divider className="mb-4" />
                <div className="flex justify-end">
                  <Pagination
                    current={storePage}
                    total={totalUsers}
                    pageSize={storePageSize}
                    showSizeChanger
                    showQuickJumper
                    showTotal={(total, range) =>
                      `${range[0]}-${range[1]} of ${total} users`
                    }
                    onChange={handlePageChange}
                    onShowSizeChange={handlePageChange}
                    pageSizeOptions={['5', '10', '20', '50']}
                  />
                </div>
              </>
            )}
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
                className={'mr-2'}
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
                      <Button type="link" danger>
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
    </SettingsPageContainer>
  )
}
