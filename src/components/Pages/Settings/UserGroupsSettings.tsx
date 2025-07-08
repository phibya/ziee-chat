import { useEffect, useState } from 'react'
import {
  Badge,
  Button,
  Card,
  Drawer,
  Form,
  Input,
  List,
  message,
  Modal,
  Popconfirm,
  Space,
  Switch,
  Table,
  Tag,
  Typography,
} from 'antd'
import {
  DeleteOutlined,
  EditOutlined,
  PlusOutlined,
  TeamOutlined,
  UserOutlined,
} from '@ant-design/icons'
import type { ColumnsType } from 'antd/es/table'
import { isDesktopApp } from '../../../api/core'
import {
  CreateUserGroupRequest,
  UpdateUserGroupRequest,
  User,
  UserGroup,
} from '../../../api/enpoints'
import { ApiClient } from '../../../api/client.ts'

const { Title, Text } = Typography
const { TextArea } = Input

export function UserGroupsSettings() {
  const [groups, setGroups] = useState<UserGroup[]>([])
  const [loading, setLoading] = useState(false)
  const [createModalVisible, setCreateModalVisible] = useState(false)
  const [editModalVisible, setEditModalVisible] = useState(false)
  const [membersDrawerVisible, setMembersDrawerVisible] = useState(false)
  const [selectedGroup, setSelectedGroup] = useState<UserGroup | null>(null)
  const [groupMembers, setGroupMembers] = useState<User[]>([])
  const [membersLoading, setMembersLoading] = useState(false)
  const [createForm] = Form.useForm()
  const [editForm] = Form.useForm()

  // Redirect if desktop app
  useEffect(() => {
    if (isDesktopApp) {
      message.warning('User group management is not available in desktop mode')
      return
    }
    fetchGroups()
  }, [])

  const fetchGroups = async () => {
    setLoading(true)
    try {
      const { groups } = await ApiClient.Admin.listGroups({
        page: 1,
        per_page: 100,
      })
      setGroups(groups)
    } catch (error) {
      message.error('Failed to fetch user groups')
    } finally {
      setLoading(false)
    }
  }

  const handleCreateGroup = async (values: any) => {
    try {
      const groupData: CreateUserGroupRequest = {
        name: values.name,
        description: values.description,
        permissions: values.permissions ? JSON.parse(values.permissions) : {},
      }
      await ApiClient.Admin.createGroup(groupData)
      message.success('User group created successfully')
      setCreateModalVisible(false)
      createForm.resetFields()
      fetchGroups()
    } catch (error) {
      message.error('Failed to create user group')
    }
  }

  const handleEditGroup = async (values: any) => {
    if (!selectedGroup) return

    try {
      const updateData: UpdateUserGroupRequest = {
        group_id: selectedGroup.id,
        name: values.name,
        description: values.description,
        permissions: values.permissions
          ? JSON.parse(values.permissions)
          : undefined,
        is_active: values.is_active,
      }
      await ApiClient.Admin.updateGroup(updateData)
      message.success('User group updated successfully')
      setEditModalVisible(false)
      setSelectedGroup(null)
      editForm.resetFields()
      fetchGroups()
    } catch (error) {
      message.error('Failed to update user group')
    }
  }

  const handleDeleteGroup = async (groupId: string) => {
    try {
      await ApiClient.Admin.deleteGroup({ group_id: groupId })
      message.success('User group deleted successfully')
      fetchGroups()
    } catch (error) {
      message.error('Failed to delete user group')
    }
  }

  const handleViewMembers = async (group: UserGroup) => {
    setSelectedGroup(group)
    setMembersDrawerVisible(true)
    setMembersLoading(true)
    try {
      const { users } = await ApiClient.Admin.getGroupMembers({
        group_id: group.id,
        page: 1,
        per_page: 100,
      })
      setGroupMembers(users)
    } catch (error) {
      message.error('Failed to fetch group members')
    } finally {
      setMembersLoading(false)
    }
  }

  const openEditModal = (group: UserGroup) => {
    setSelectedGroup(group)
    editForm.setFieldsValue({
      name: group.name,
      description: group.description,
      permissions: JSON.stringify(group.permissions, null, 2),
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
        <Space>
          <TeamOutlined />
          <span>{name}</span>
          {!record.is_active && <Tag color="red">Inactive</Tag>}
        </Space>
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
        <Space>
          <Button
            type="link"
            icon={<UserOutlined />}
            onClick={() => handleViewMembers(record)}
          >
            Members
          </Button>
          <Button
            type="link"
            icon={<EditOutlined />}
            onClick={() => openEditModal(record)}
          >
            Edit
          </Button>
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
        </Space>
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

  return (
    <div>
      <div className="flex justify-between items-center mb-6">
        <Title level={3}>User Groups</Title>
        <Button
          type="primary"
          icon={<PlusOutlined />}
          onClick={() => setCreateModalVisible(true)}
        >
          Create Group
        </Button>
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
      <Modal
        title="Create User Group"
        open={createModalVisible}
        onCancel={() => {
          setCreateModalVisible(false)
          createForm.resetFields()
        }}
        footer={null}
        width={600}
      >
        <Form form={createForm} layout="vertical" onFinish={handleCreateGroup}>
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
          <Form.Item className="mb-0">
            <Space>
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
            </Space>
          </Form.Item>
        </Form>
      </Modal>

      {/* Edit Group Modal */}
      <Modal
        title="Edit User Group"
        open={editModalVisible}
        onCancel={() => {
          setEditModalVisible(false)
          setSelectedGroup(null)
          editForm.resetFields()
        }}
        footer={null}
        width={600}
      >
        <Form form={editForm} layout="vertical" onFinish={handleEditGroup}>
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
            <TextArea rows={6} />
          </Form.Item>
          <Form.Item name="is_active" label="Active" valuePropName="checked">
            <Switch />
          </Form.Item>
          <Form.Item className="mb-0">
            <Space>
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
            </Space>
          </Form.Item>
        </Form>
      </Modal>

      {/* Group Members Drawer */}
      <Drawer
        title={`Members of ${selectedGroup?.name}`}
        placement="right"
        onClose={() => setMembersDrawerVisible(false)}
        open={membersDrawerVisible}
        width={400}
      >
        <List
          loading={membersLoading}
          dataSource={groupMembers}
          renderItem={user => (
            <List.Item>
              <List.Item.Meta
                avatar={<UserOutlined />}
                title={user.username}
                description={
                  <div>
                    <div>{user.emails[0]?.address}</div>
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
  )
}
