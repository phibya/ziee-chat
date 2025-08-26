import React, { useEffect } from 'react'
import { Button, Form, Input, Select } from 'antd'
import { Drawer } from '../../common/Drawer.tsx'
import {
  closeRAGInstanceDrawer,
  createRAGInstance,
  setRAGInstanceDrawerLoading,
  Stores,
  updateRAGInstanceInList,
} from '../../../store'
import { PermissionGuard } from '../../Auth/PermissionGuard.tsx'
import { Permission } from '../../../types'

const { TextArea } = Input

interface RAGInstanceFormData {
  name: string
  description?: string
  provider_id: string
}

export const RagFormDrawer: React.FC = () => {
  const [form] = Form.useForm<RAGInstanceFormData>()

  // Store usage
  const { open, loading, editingInstance } = Stores.UI.RAGInstanceDrawer
  const { creatableProviders } = Stores.RAG

  // System instance permission checks
  const isSystemInstance = editingInstance?.is_system

  const handleSubmit = async (values: RAGInstanceFormData) => {
    const finalValues = {
      ...values,
      description: values.description || '',
    }

    setRAGInstanceDrawerLoading(true)
    try {
      if (editingInstance) {
        await updateRAGInstanceInList(editingInstance.id, finalValues)
      } else {
        await createRAGInstance({
          name: finalValues.name,
          provider_id: values.provider_id,
          alias: finalValues.name.toLowerCase().replace(/\s+/g, '_'),
          description: finalValues.description,
          engine_type: 'simple_vector', // Default to vector engine
        })
      }
      closeRAGInstanceDrawer()
    } catch (error) {
      console.error('Failed to save RAG instance:', error)
    } finally {
      setRAGInstanceDrawerLoading(false)
    }
  }

  // Initialize form when drawer opens or editing instance changes
  useEffect(() => {
    if (open) {
      if (editingInstance) {
        // Editing existing instance
        form.setFieldsValue({
          name: editingInstance.name,
          description: editingInstance.description,
          provider_id: editingInstance.provider_id,
        })
      } else {
        // Creating new instance - no default values needed
      }
    } else {
      // Reset when drawer closes
      form.resetFields()
    }
  }, [open, editingInstance, form])

  const getTitle = () => {
    if (editingInstance) {
      return `RAG Instance: ${editingInstance.name}`
    }
    return 'Create RAG Instance'
  }

  return (
    <Drawer
      title={getTitle()}
      open={open}
      onClose={closeRAGInstanceDrawer}
      footer={[
        <Button key="cancel" onClick={closeRAGInstanceDrawer} disabled={loading}>
          Cancel
        </Button>,
        <PermissionGuard
          permissions={[
            editingInstance
              ? (isSystemInstance ? Permission.RagAdminInstancesEdit : Permission.RagInstancesEdit)
              : Permission.RagInstancesCreate,
          ]}
          type={'disabled'}
        >
          <Button
            key="submit"
            type="primary"
            loading={loading}
            onClick={() => form.submit()}
          >
            {editingInstance ? 'Update' : 'Create'}
          </Button>
        </PermissionGuard>,
      ]}
      width={400}
      maskClosable={false}
    >
      
      <PermissionGuard
        permissions={[
          editingInstance 
            ? (isSystemInstance ? Permission.RagAdminInstancesEdit : Permission.RagInstancesEdit)
            : Permission.RagInstancesCreate,
        ]}
        type={'disabled'}
      >
        <Form 
          form={form} 
          onFinish={handleSubmit} 
          layout="vertical"
        >
          <Form.Item
            name="name"
            label="Instance Name"
            rules={[{ required: true, message: 'Please enter an instance name' }]}
          >
            <Input placeholder="Enter instance name" />
          </Form.Item>

          <Form.Item name="description" label="Description">
            <TextArea placeholder="Enter instance description" rows={4} />
          </Form.Item>

          {!editingInstance && (
            <Form.Item
              name="provider_id"
              label="RAG Provider"
              rules={[{ required: true, message: 'Please select a provider' }]}
            >
              <Select placeholder="Select a RAG provider">
                {creatableProviders.map(provider => (
                  <Select.Option 
                    key={provider.id} 
                    value={provider.id}
                    disabled={!provider.enabled}
                  >
                    {provider.name}
                  </Select.Option>
                ))}
              </Select>
            </Form.Item>
          )}
        </Form>
      </PermissionGuard>
    </Drawer>
  )
}