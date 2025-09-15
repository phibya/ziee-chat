import {
  DeleteOutlined,
  EditOutlined,
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
  Select,
  Spin,
  Tag,
  Typography,
} from 'antd'
import { Drawer } from '../../../common/Drawer.tsx'
import { useEffect, useState } from 'react'
import { isTauriView } from '../../../../api/core.ts'
import {
  clearSystemAdminError,
  createNewUserGroup,
  deleteUserGroup,
  getGroupProviders,
  getGroupRagProviders,
  loadUserGroupMembers,
  loadUserGroups,
  Stores,
} from '../../../../store'
import { CreateUserGroupRequest, UserGroup } from '../../../../types'
import { SettingsPageContainer } from '../common/SettingsPageContainer.tsx'
import { EditUserGroupDrawer } from './EditUserGroupDrawer.tsx'

const { Title, Text } = Typography
const { TextArea } = Input

export function UserGroupsSettings() {
  const { message } = App.useApp()

  const {
    groups,
    currentGroupMembers,
    total: totalGroups,
    currentPage: storePage,
    pageSize: storePageSize,
    loadingGroups,
    loadingGroupMembers,
    error,
  } = Stores.AdminUserGroups
  const { providers: providers } = Stores.AdminProviders
  const { providers: ragProviders } = Stores.AdminRAGProviders

  const [createModalVisible, setCreateModalVisible] = useState(false)
  const [editModalVisible, setEditModalVisible] = useState(false)
  const [membersDrawerVisible, setMembersDrawerVisible] = useState(false)
  const [selectedGroup, setSelectedGroup] = useState<UserGroup | null>(null)
  const [createForm] = Form.useForm()

  // Store providers for each group
  const [groupProviders, setGroupProviders] = useState<Record<string, any[]>>(
    {},
  )
  const [groupRagProviders, setGroupRagProviders] = useState<
    Record<string, any[]>
  >({})

  // Show errors
  useEffect(() => {
    if (error) {
      message.error(error)
      clearSystemAdminError()
    }
  }, [error, message])

  // Load providers for all groups when groups are loaded
  useEffect(() => {
    const loadGroupProviders = async () => {
      if (groups.length > 0) {
        const providersMap: Record<string, any[]> = {}
        const ragProvidersMap: Record<string, any[]> = {}

        for (const group of groups) {
          try {
            const [providersResponse, ragProvidersResponse] = await Promise.all(
              [getGroupProviders(group.id), getGroupRagProviders(group.id)],
            )
            providersMap[group.id] = providersResponse.providers
            ragProvidersMap[group.id] = ragProvidersResponse.providers
          } catch (error) {
            console.error(
              `Failed to load providers for group ${group.id}:`,
              error,
            )
            providersMap[group.id] = []
            ragProvidersMap[group.id] = []
          }
        }

        setGroupProviders(providersMap)
        setGroupRagProviders(ragProvidersMap)
      }
    }

    loadGroupProviders()
  }, [groups])

  const handleCreateGroup = async (values: any) => {
    // Check if user is trying to assign model providers but doesn't have permission
    if (values.provider_ids && values.provider_ids.length > 0) {
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
        rag_provider_ids: values.rag_provider_ids || [],
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

  const handleEditSuccess = () => {
    setEditModalVisible(false)
    setSelectedGroup(null)
    // Reload providers for the group list
    const loadGroupProviders = async () => {
      if (groups.length > 0) {
        const providersMap: Record<string, any[]> = {}
        const ragProvidersMap: Record<string, any[]> = {}

        for (const group of groups) {
          try {
            const [providersResponse, ragProvidersResponse] = await Promise.all(
              [getGroupProviders(group.id), getGroupRagProviders(group.id)],
            )
            providersMap[group.id] = providersResponse.providers
            ragProvidersMap[group.id] = ragProvidersResponse.providers
          } catch (error) {
            console.error(
              `Failed to load providers for group ${group.id}:`,
              error,
            )
            providersMap[group.id] = []
            ragProvidersMap[group.id] = []
          }
        }

        setGroupProviders(providersMap)
        setGroupRagProviders(ragProvidersMap)
      }
    }

    loadGroupProviders()
  }

  const handleDeleteGroup = async (groupId: string) => {
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
    setEditModalVisible(true)
  }

  const getGroupActions = (group: UserGroup) => {
    const actions: React.ReactNode[] = []

    actions.push(
      <Button
        key="members"
        type="text"
        icon={<UserOutlined />}
        onClick={() => handleViewMembers(group)}
      >
        Members
      </Button>,
    )

    actions.push(
      <Button
        key="edit"
        type="text"
        icon={<EditOutlined />}
        onClick={() => openEditModal(group)}
      >
        Edit
      </Button>,
    )

    actions.push(
      <Popconfirm
        key="delete"
        title="Are you sure you want to delete this group?"
        onConfirm={() => handleDeleteGroup(group.id)}
        okText="Yes"
        cancelText="No"
      >
        <Button type="text" danger icon={<DeleteOutlined />}>
          Delete
        </Button>
      </Popconfirm>,
    )

    return actions.filter(Boolean)
  }

  const handlePageChange = (page: number, size?: number) => {
    const newPageSize = size || storePageSize
    const newPage = size && size !== storePageSize ? 1 : page // Reset to page 1 if page size changes

    loadUserGroups(newPage, newPageSize)
  }

  if (isTauriView) {
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
    <SettingsPageContainer title="User Groups">
      <div>
        <Card
          title="User Groups"
          extra={
            <Button
              type="primary"
              icon={<PlusOutlined />}
              onClick={() => setCreateModalVisible(true)}
            >
              Create Group
            </Button>
          }
        >
          {loadingGroups ? (
            <div className="flex justify-center py-8">
              <Spin size="large" />
            </div>
          ) : groups.length === 0 ? (
            <div>
              <Empty description="No user groups found" />
            </div>
          ) : (
            <div>
              {groups.map((group, index) => (
                <div key={group.id}>
                  <div className="flex items-start gap-3 flex-wrap">
                    {/* Group Info */}
                    <div className="flex-1">
                      <div className="flex items-center gap-2 mb-2 flex-wrap">
                        <div className={'flex-1 min-w-48'}>
                          <Flex className="gap-2 items-center">
                            <TeamOutlined />
                            <Text className="font-medium">{group.name}</Text>
                            {group.is_protected && (
                              <Tag color="orange">Protected</Tag>
                            )}
                            <Badge
                              status={group.is_active ? 'success' : 'error'}
                              text={group.is_active ? 'Active' : 'Inactive'}
                            />
                          </Flex>
                        </div>
                        <div className={'flex gap-1 items-center justify-end'}>
                          {getGroupActions(group)}
                        </div>
                      </div>

                      <Descriptions
                        size="small"
                        column={{ xs: 1, sm: 2, md: 3 }}
                        colon={false}
                        labelStyle={{ fontSize: '12px', color: '#8c8c8c' }}
                        contentStyle={{ fontSize: '12px' }}
                      >
                        <Descriptions.Item label="Description">
                          {group.description || 'No description'}
                        </Descriptions.Item>
                        <Descriptions.Item label="Permissions">
                          <Text code>
                            {Object.keys(group.permissions || {}).length}{' '}
                            permissions
                          </Text>
                        </Descriptions.Item>
                        <Descriptions.Item label="Created">
                          {new Date(group.created_at).toLocaleDateString()}
                        </Descriptions.Item>
                        {groupProviders[group.id] &&
                          groupProviders[group.id].length > 0 && (
                            <Descriptions.Item
                              label="Model Providers"
                              span={{ xs: 1, sm: 2, md: 3 }}
                            >
                              <Flex wrap className="gap-1">
                                {groupProviders[group.id].map(
                                  (provider: any) => {
                                    return (
                                      <Tag
                                        key={provider.id}
                                        color="blue"
                                        className="text-xs"
                                      >
                                        {provider.name}
                                      </Tag>
                                    )
                                  },
                                )}
                              </Flex>
                            </Descriptions.Item>
                          )}
                        {groupRagProviders[group.id] &&
                          groupRagProviders[group.id].length > 0 && (
                            <Descriptions.Item
                              label="RAG Providers"
                              span={{ xs: 1, sm: 2, md: 3 }}
                            >
                              <Flex wrap className="gap-1">
                                {groupRagProviders[group.id].map(
                                  (provider: any) => {
                                    return (
                                      <Tag
                                        key={provider.id}
                                        color="green"
                                        className="text-xs"
                                      >
                                        {provider.name}
                                      </Tag>
                                    )
                                  },
                                )}
                              </Flex>
                            </Descriptions.Item>
                          )}
                      </Descriptions>
                    </div>
                  </div>
                  {index < groups.length - 1 && <Divider className="my-0" />}
                </div>
              ))}
            </div>
          )}

          {groups.length > 0 && (
            <>
              <Divider className="mb-4" />
              <div className="flex justify-end">
                <Pagination
                  current={storePage}
                  total={totalGroups}
                  pageSize={storePageSize}
                  showSizeChanger
                  showQuickJumper
                  showTotal={(total, range) =>
                    `${range[0]}-${range[1]} of ${total} groups`
                  }
                  onChange={handlePageChange}
                  onShowSizeChange={handlePageChange}
                  pageSizeOptions={['5', '10', '20', '50']}
                />
              </div>
            </>
          )}
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

            <Form.Item
              name="provider_ids"
              label="Model Providers"
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
            <Form.Item
              name="rag_provider_ids"
              label="RAG Providers"
              tooltip="Select which RAG providers this group can access"
            >
              <Select
                mode="multiple"
                placeholder="Select RAG providers"
                options={ragProviders.map(provider => ({
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

        {/* Edit Group Drawer */}
        <EditUserGroupDrawer
          group={selectedGroup}
          open={editModalVisible}
          onClose={() => {
            setEditModalVisible(false)
            setSelectedGroup(null)
          }}
          onSuccess={handleEditSuccess}
        />

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
    </SettingsPageContainer>
  )
}
