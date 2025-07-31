import { Button, Drawer, Form, Input, InputNumber, Switch } from 'antd'
import { useState } from 'react'
import { updateRAGRepository } from '../../../../store'

interface EditRAGRepositoryDrawerProps {
  open?: boolean
  onClose?: () => void
  repository?: any
}

export function EditRAGRepositoryDrawer({ 
  open = false, 
  onClose, 
  repository 
}: EditRAGRepositoryDrawerProps) {
  const [form] = Form.useForm()
  const [loading, setLoading] = useState(false)

  const handleSubmit = async (values: any) => {
    if (!repository) return

    setLoading(true)
    try {
      await updateRAGRepository(repository.id, {
        name: values.name,
        description: values.description,
        url: values.url,
        enabled: values.enabled,
        requires_auth: values.requires_auth,
        auth_token: values.auth_token,
        priority: values.priority,
      })
      onClose?.()
    } catch (error) {
      console.error('Failed to update RAG repository:', error)
    } finally {
      setLoading(false)
    }
  }

  const handleClose = () => {
    form.resetFields()
    onClose?.()
  }

  return (
    <Drawer
      title="Edit RAG Repository"
      width={400}
      open={open}
      onClose={handleClose}
      footer={
        <div style={{ textAlign: 'right' }}>
          <Button onClick={handleClose} style={{ marginRight: 8 }}>
            Cancel
          </Button>
          <Button
            type="primary"
            loading={loading}
            onClick={() => form.submit()}
          >
            Save Changes
          </Button>
        </div>
      }
    >
      <Form
        form={form}
        layout="vertical"
        onFinish={handleSubmit}
        initialValues={repository}
      >
        <Form.Item
          label="Name"
          name="name"
          rules={[{ required: true, message: 'Please enter a name' }]}
        >
          <Input placeholder="Enter repository name" />
        </Form.Item>

        <Form.Item
          label="Description"
          name="description"
        >
          <Input.TextArea placeholder="Enter description" rows={3} />
        </Form.Item>

        <Form.Item
          label="URL"
          name="url"
          rules={[
            { required: true, message: 'Please enter a URL' },
            { type: 'url', message: 'Please enter a valid URL' },
          ]}
        >
          <Input placeholder="https://example.com/rag-repository" />
        </Form.Item>

        <Form.Item
          label="Priority"
          name="priority"
          tooltip="Higher priority repositories are checked first"
        >
          <InputNumber min={0} max={100} style={{ width: '100%' }} />
        </Form.Item>

        <Form.Item
          label="Requires Authentication"
          name="requires_auth"
          valuePropName="checked"
        >
          <Switch />
        </Form.Item>

        <Form.Item
          label="Auth Token"
          name="auth_token"
          dependencies={['requires_auth']}
          rules={[
            ({ getFieldValue }) => ({
              required: getFieldValue('requires_auth'),
              message: 'Please enter an auth token',
            }),
          ]}
        >
          <Input.Password placeholder="Enter authentication token" />
        </Form.Item>

        <Form.Item
          label="Enabled"
          name="enabled"
          valuePropName="checked"
        >
          <Switch />
        </Form.Item>
      </Form>
    </Drawer>
  )
}