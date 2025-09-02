import { App, Button, Form, Input, Select, Switch, Typography } from 'antd'
import { Drawer } from '../../../common/Drawer.tsx'
import { useEffect, useState } from 'react'
import {
  getGroupProviders,
  getGroupRagProviders,
  Stores,
  updateUserGroup,
} from '../../../../store'
import { UpdateUserGroupRequest, UserGroup } from '../../../../types'

const { TextArea } = Input
const { Text } = Typography

interface EditUserGroupDrawerProps {
  group: UserGroup | null
  open: boolean
  onClose: () => void
  onSuccess?: () => void
}

export function EditUserGroupDrawer({
  group,
  open,
  onClose,
  onSuccess,
}: EditUserGroupDrawerProps) {
  const { message } = App.useApp()
  const [form] = Form.useForm()
  const [loading, setLoading] = useState(false)
  const [loadingProviders, setLoadingProviders] = useState(false)
  
  const { providers } = Stores.AdminProviders
  const { providers: ragProviders } = Stores.AdminRAGProviders
  const { updating } = Stores.AdminUserGroups

  // Load providers for the group when it changes
  useEffect(() => {
    if (group && open) {
      const loadGroupData = async () => {
        setLoadingProviders(true)
        try {
          // Fetch providers for this group
          const [providersResponse, ragProvidersResponse] = await Promise.all([
            getGroupProviders(group.id),
            getGroupRagProviders(group.id)
          ])
          
          const providerIds = providersResponse.providers.map((p: any) => p.id)
          const ragProviderIds = ragProvidersResponse.providers.map((p: any) => p.id)
          
          form.setFieldsValue({
            name: group.name,
            description: group.description,
            permissions: JSON.stringify(group.permissions, null, 2),
            provider_ids: providerIds,
            rag_provider_ids: ragProviderIds,
            is_active: group.is_active,
          })
        } catch (error) {
          console.error('Failed to load group providers:', error)
          form.setFieldsValue({
            name: group.name,
            description: group.description,
            permissions: JSON.stringify(group.permissions, null, 2),
            provider_ids: [],
            rag_provider_ids: [],
            is_active: group.is_active,
          })
          message.error('Failed to load group providers')
        } finally {
          setLoadingProviders(false)
        }
      }

      loadGroupData()
    }
  }, [group, open, form, message])

  const handleClose = () => {
    form.resetFields()
    onClose()
  }

  const handleSubmit = async (values: any) => {
    if (!group) return

    try {
      setLoading(true)

      let permissions: string[] = []
      try {
        permissions = JSON.parse(values.permissions || '[]')
        if (!Array.isArray(permissions)) {
          throw new Error('Permissions must be an array')
        }
      } catch (error) {
        message.error('Invalid permissions format. Please enter a valid JSON array.')
        return
      }

      const updateData: UpdateUserGroupRequest = {
        name: values.name,
        description: values.description,
        permissions,
        provider_ids: values.provider_ids || [],
        rag_provider_ids: values.rag_provider_ids || [],
        is_active: values.is_active,
      }

      await updateUserGroup(group.id, updateData)
      message.success('User group updated successfully')
      handleClose()
      onSuccess?.()
    } catch (error) {
      console.error('Failed to update user group:', error)
      message.error('Failed to update user group')
    } finally {
      setLoading(false)
    }
  }

  return (
    <Drawer
      title="Edit User Group"
      open={open}
      onClose={handleClose}
      footer={null}
      width={600}
      maskClosable={false}
    >
      <Form
        form={form}
        layout="vertical"
        onFinish={handleSubmit}
        disabled={loadingProviders}
      >
        <Form.Item
          name="name"
          label="Group Name"
          rules={[
            { required: true, message: 'Please enter a group name' },
            { min: 2, message: 'Group name must be at least 2 characters' },
          ]}
        >
          <Input placeholder="Enter group name" />
        </Form.Item>

        <Form.Item name="description" label="Description">
          <TextArea
            placeholder="Enter group description (optional)"
            rows={3}
            showCount
            maxLength={500}
          />
        </Form.Item>

        <Form.Item
          name="permissions"
          label="Permissions"
          rules={[
            {
              validator: async (_, value) => {
                if (value) {
                  try {
                    const parsed = JSON.parse(value)
                    if (!Array.isArray(parsed)) {
                      throw new Error('Must be an array')
                    }
                  } catch (error) {
                    throw new Error('Invalid JSON format')
                  }
                }
              },
            },
          ]}
        >
          <TextArea
            placeholder='["permission1", "permission2"]'
            rows={4}
          />
        </Form.Item>

        <Form.Item name="provider_ids" label="Model Providers">
          <Select
            mode="multiple"
            placeholder="Select model providers"
            options={providers.map(provider => ({
              label: provider.name,
              value: provider.id,
            }))}
            showSearch
            filterOption={(input, option) =>
              (option?.label ?? '').toLowerCase().includes(input.toLowerCase())
            }
          />
        </Form.Item>

        <Form.Item name="rag_provider_ids" label="RAG Providers">
          <Select
            mode="multiple"
            placeholder="Select RAG providers"
            options={ragProviders.map(provider => ({
              label: provider.name,
              value: provider.id,
            }))}
            showSearch
            filterOption={(input, option) =>
              (option?.label ?? '').toLowerCase().includes(input.toLowerCase())
            }
          />
        </Form.Item>

        <Form.Item name="is_active" label="Active" valuePropName="checked">
          <Switch />
        </Form.Item>

        <div className="flex justify-end gap-3 pt-4">
          <Button onClick={handleClose} disabled={loading || updating}>
            Cancel
          </Button>
          <Button
            type="primary"
            htmlType="submit"
            loading={loading || updating || loadingProviders}
            disabled={loadingProviders}
          >
            {loadingProviders ? 'Loading...' : 'Update Group'}
          </Button>
        </div>
      </Form>

      {loadingProviders && (
        <div className="mt-4">
          <Text type="secondary">Loading group providers...</Text>
        </div>
      )}
    </Drawer>
  )
}