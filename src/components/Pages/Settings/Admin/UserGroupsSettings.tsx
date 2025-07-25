import {
  DeleteOutlined,
  EditOutlined,
  ExclamationCircleOutlined,
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
import { isDesktopApp } from '../../../../api/core.ts'
import { Permission, usePermissions } from '../../../../permissions'
import {
  clearSystemAdminError,
  createNewUserGroup,
  deleteUserGroup,
  loadAllModelProviders,
  loadAllUserGroups,
  loadUserGroupMembers,
  Stores,
  updateUserGroup,
} from '../../../../store'
import {
  CreateUserGroupRequest,
  UpdateUserGroupRequest,
  UserGroup,
} from '../../../../types'
import { PageContainer } from '../../../common/PageContainer'

const { Title, Text } = Typography
const { TextArea } = Input

export function UserGroupsSettings() {
  const { message } = App.useApp()
  const { hasPermission } = usePermissions()

  const { groups, currentGroupMembers, loading, loadingGroupMembers, error } =
    Stores.Admin
  const { providers: providers } = Stores.Providers

  const [createModalVisible, setCreateModalVisible] = useState(false)
  const [editModalVisible, setEditModalVisible] = useState(false)
  const [membersDrawerVisible, setMembersDrawerVisible] = useState(false)
  const [selectedGroup, setSelectedGroup] = useState<UserGroup | null>(null)
  const [createForm] = Form.useForm()
  const [editForm] = Form.useForm()

  // Check permissions
  const canReadGroups = hasPermission(Permission.groups.read)
  const canEditGroups = hasPermission(Permission.groups.edit)
  const canCreateGroups = hasPermission(Permission.groups.create)
  const canDeleteGroups = hasPermission(Permission.groups.delete)
  const canManageProviders = hasPermission(Permission.config.providers.edit)

  // Redirect if desktop app or insufficient permissions
  useEffect(() => {
    if (isDesktopApp) {
      message.warning('User group management is not available in desktop mode')
      return
    }
    if (!canReadGroups) {
      message.warning('You do not have permission to view user groups')
      return
    }
    loadAllUserGroups()
    loadAllModelProviders()
  }, [canReadGroups])

  // Show errors
  useEffect(() => {
    if (error) {
      message.error(error)
      clearSystemAdminError()
    }
  }, [error, message])

  const handleCreateGroup = async (values: any) => {
    if (!canCreateGroups) {
      message.error('You do not have permission to create user groups')
      return
    }

    // Check if user is trying to assign model providers but doesn't have permission
    if (
      values.provider_ids &&
      values.provider_ids.length > 0 &&
      !canManageProviders
    ) {
      message.error(
        'You do not have permission to assign model providers to groups',
      )
      return
    }

    try {
      const groupData: CreateUserGroupRequest = {
        name: values.name,
        description: values.description,
        permissions: values.permissions ? JSON.parse(values.permissions) : {},
        provider_ids: values.provider_ids || [],
      }
      await createNewUserGroup(groupData)
      message.success('User group created successfully')
      setCreateModalVisible(false)
      createForm.resetFields()
    } catch (error) {
      console.error('Failed to create user group:', error)
      // Error is handled by the store
    }
  }

  const handleEditGroup = async (values: any) => {
    if (!selectedGroup) return
    if (!canEditGroups) {
      message.error('You do not have permission to edit user groups')
      return
    }

    // Check if user is trying to modify model providers but doesn't have permission
    const originalProviders = selectedGroup.provider_ids || []
    const newProviders = values.provider_ids || []
    const providersChanged =
      JSON.stringify(originalProviders.sort()) !==
      JSON.stringify(newProviders.sort())

    if (providersChanged && !canManageProviders) {
      message.error(
        'You do not have permission to modify model provider assignments',
      )
      return
    }

    try {
      const updateData: UpdateUserGroupRequest = {
        group_id: selectedGroup.id,
        name: selectedGroup.is_protected ? undefined : values.name,
        description: values.description,
        permissions: selectedGroup.is_protected
          ? undefined
          : values.permissions
            ? JSON.parse(values.permissions)
            : undefined,
        provider_ids: values.provider_ids || [],
        is_active: selectedGroup.is_protected ? undefined : values.is_active,
      }
      await updateUserGroup(selectedGroup.id, updateData)
      message.success('User group updated successfully')
      setEditModalVisible(false)
      setSelectedGroup(null)
      editForm.resetFields()
    } catch (error) {
      console.error('Failed to update user group:', error)
      // Error is handled by the store
    }
  }

  const handleDeleteGroup = async (groupId: string) => {
    if (!canDeleteGroups) {
      message.error('You do not have permission to delete user groups')
      return
    }
    try {
      await deleteUserGroup(groupId)
      message.success('User group deleted successfully')
    } catch (error) {
      console.error('Failed to delete user group:', error)
      // Error is handled by the store
    }
  }

  const handleViewMembers = async (group: UserGroup) => {
    setSelectedGroup(group)
    setMembersDrawerVisible(true)

    try {
      await loadUserGroupMembers(group.id)
    } catch (error) {
      console.error('Failed to fetch group members:', error)
      // Error is handled by the store
    }
  }

  const openEditModal = (group: UserGroup) => {
    setSelectedGroup(group)
    editForm.setFieldsValue({
      name: group.name,
      description: group.description,
      permissions: JSON.stringify(group.permissions, null, 2),
      provider_ids: group.provider_ids || [],
      is_active: group.is_active,
    })
    setEditModalVisible(true)
  }

  const columns: ColumnsType<UserGroup> = [
    {
      title: 'Name',
      dataIndex: 'name',
      key: 'name',
      render: (name: string, record: UserGroup) => (
        <Flex className="gap-2">
          <TeamOutlined />
          <span>{name}</span>
          {record.is_protected && <Tag color="orange">Protected</Tag>}
          {!record.is_active && <Tag color="red">Inactive</Tag>}
        </Flex>
      ),
    },
    {
      title: 'Description',
      dataIndex: 'description',
      key: 'description',
      render: (desc: string) =>
        desc || <Text type="secondary">No description</Text>,
    },
    {
      title: 'Permissions',
      dataIndex: 'permissions',
      key: 'permissions',
      render: (permissions: any) => (
        <Text code>{Object.keys(permissions || {}).length} permissions</Text>
      ),
    },
    ...(canManageProviders
      ? [
          {
            title: 'Providers',
            dataIndex: 'provider_ids',
            key: 'provider_ids',
            render: (providerIds: string[], record: UserGroup) => {
              const ids = providerIds || record.provider_ids || []
              if (ids.length === 0) {
                return <Text type="secondary">No providers assigned</Text>
              }
              return (
                <Flex wrap className="gap-1">
                  {ids.map(providerId => {
                    const provider = providers.find(p => p.id === providerId)
                    return (
                      <Tag key={providerId} color="blue">
                        {provider?.name || providerId}
                      </Tag>
                    )
                  })}
                </Flex>
              )
            },
          },
        ]
      : []),
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
      title: 'Created',
      dataIndex: 'created_at',
      key: 'created_at',
      render: (date: string) => new Date(date).toLocaleDateString(),
    },
    {
      title: 'Actions',
      key: 'actions',
      render: (_, record: UserGroup) => (
        <Flex className="gap-2">
          <Button
            type="link"
            icon={<UserOutlined />}
            onClick={() => handleViewMembers(record)}
          >
            Members
          </Button>
          {canEditGroups && (
            <Button
              type="link"
              icon={<EditOutlined />}
              onClick={() => openEditModal(record)}
            >
              Edit
            </Button>
          )}
          {canDeleteGroups && !record.is_protected && (
            <Popconfirm
              title="Are you sure you want to delete this group?"
              onConfirm={() => handleDeleteGroup(record.id)}
              okText="Yes"
              cancelText="No"
            >
              <Button type="link" danger icon={<DeleteOutlined />}>
                Delete
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
          <Title level={4}>User Group Management</Title>
          <Text type="secondary">
            User group management is disabled in desktop mode
          </Text>
        </div>
      </Card>
    )
  }

  if (!canReadGroups) {
    return (
      <Result
        icon={<ExclamationCircleOutlined />}
        title="Access Denied"
        subTitle={`You do not have permission to view user groups. Contact your administrator to request ${Permission.groups.read} permission.`}
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
          <Title level={3}>User Groups</Title>
          {canCreateGroups && (
            <Button
              type="primary"
              icon={<PlusOutlined />}
              onClick={() => setCreateModalVisible(true)}
            >
              Create Group
            </Button>
          )}
        </div>

        <Card>
          <Table
            columns={columns}
            dataSource={groups}
            rowKey="id"
            loading={loading}
            pagination={{
              pageSize: 10,
              showSizeChanger: true,
              showTotal: total => `Total ${total} groups`,
            }}
          />
        </Card>

        {/* Create Group Modal */}
        <Drawer
          title="Create User Group"
          open={createModalVisible}
          onClose={() => {
            setCreateModalVisible(false)
            createForm.resetFields()
          }}
          footer={null}
          width={600}
          maskClosable={false}
        >
          <Form
            form={createForm}
            layout="vertical"
            onFinish={handleCreateGroup}
          >
            <Form.Item
              name="name"
              label="Group Name"
              rules={[{ required: true, message: 'Please enter group name' }]}
            >
              <Input placeholder="Enter group name" />
            </Form.Item>
            <Form.Item name="description" label="Description">
              <TextArea rows={3} placeholder="Enter group description" />
            </Form.Item>
            <Form.Item
              name="permissions"
              label="Permissions (JSON)"
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
              <TextArea
                rows={6}
                placeholder='{"user_management": true, "chat": true}'
              />
            </Form.Item>

            {canManageProviders && (
              <Form.Item
                name="provider_ids"
                label="Providers"
                tooltip="Select which model providers this group can access"
              >
                <Select
                  mode="multiple"
                  placeholder="Select model providers"
                  options={providers.map(provider => ({
                    value: provider.id,
                    label: provider.name,
                    disabled: !provider.enabled,
                  }))}
                  showSearch
                  filterOption={(input, option) =>
                    (option?.label ?? '')
                      .toLowerCase()
                      .includes(input.toLowerCase())
                  }
                />
              </Form.Item>
            )}
            <Form.Item className="mb-0">
              <Flex className="gap-2">
                <Button type="primary" htmlType="submit">
                  Create Group
                </Button>
                <Button
                  onClick={() => {
                    setCreateModalVisible(false)
                    createForm.resetFields()
                  }}
                >
                  Cancel
                </Button>
              </Flex>
            </Form.Item>
          </Form>
        </Drawer>

        {/* Edit Group Modal */}
        <Drawer
          title="Edit User Group"
          open={editModalVisible}
          onClose={() => {
            setEditModalVisible(false)
            setSelectedGroup(null)
            editForm.resetFields()
          }}
          footer={null}
          width={600}
          maskClosable={false}
        >
          <Form form={editForm} layout="vertical" onFinish={handleEditGroup}>
            <Form.Item
              name="name"
              label="Group Name"
              tooltip={
                selectedGroup?.is_protected
                  ? 'Protected groups cannot have their name changed'
                  : undefined
              }
              rules={[{ required: true, message: 'Please enter group name' }]}
            >
              <Input
                placeholder="Enter group name"
                disabled={selectedGroup?.is_protected}
              />
            </Form.Item>
            <Form.Item name="description" label="Description">
              <TextArea rows={3} placeholder="Enter group description" />
            </Form.Item>
            <Form.Item
              name="permissions"
              label="Permissions (JSON)"
              tooltip={
                selectedGroup?.is_protected
                  ? 'Protected groups cannot have their permissions modified'
                  : undefined
              }
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
              <TextArea rows={6} disabled={selectedGroup?.is_protected} />
            </Form.Item>

            {canManageProviders && (
              <Form.Item
                name="provider_ids"
                label="Providers"
                tooltip="Select which model providers this group can access"
              >
                <Select
                  mode="multiple"
                  placeholder="Select model providers"
                  options={providers.map(provider => ({
                    value: provider.id,
                    label: provider.name,
                    disabled: !provider.enabled,
                  }))}
                  showSearch
                  filterOption={(input, option) =>
                    (option?.label ?? '')
                      .toLowerCase()
                      .includes(input.toLowerCase())
                  }
                />
              </Form.Item>
            )}

            <Form.Item
              name="is_active"
              label="Active"
              valuePropName="checked"
              tooltip={
                selectedGroup?.is_protected
                  ? 'Protected groups cannot have their active status changed'
                  : undefined
              }
            >
              <Switch disabled={selectedGroup?.is_protected} />
            </Form.Item>
            <Form.Item className="mb-0">
              <Flex className="gap-2">
                <Button type="primary" htmlType="submit">
                  Update Group
                </Button>
                <Button
                  onClick={() => {
                    setEditModalVisible(false)
                    setSelectedGroup(null)
                    editForm.resetFields()
                  }}
                >
                  Cancel
                </Button>
              </Flex>
            </Form.Item>
          </Form>
        </Drawer>

        {/* Group Members Drawer */}
        <Drawer
          title={`Members of ${selectedGroup?.name}`}
          placement="right"
          onClose={() => setMembersDrawerVisible(false)}
          open={membersDrawerVisible}
          width={400}
        >
          <List
            loading={loadingGroupMembers}
            dataSource={currentGroupMembers}
            renderItem={user => (
              <List.Item>
                <List.Item.Meta
                  avatar={<UserOutlined />}
                  title={user.username}
                  description={
                    <div>
                      <div>{user.email}</div>
                      <Tag color={user.is_active ? 'green' : 'red'}>
                        {user.is_active ? 'Active' : 'Inactive'}
                      </Tag>
                    </div>
                  }
                />
              </List.Item>
            )}
          />
        </Drawer>
      </div>
    </PageContainer>
  )
}
