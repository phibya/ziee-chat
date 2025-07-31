import { Button, Drawer, Form, Input, Select } from 'antd'
import { useState } from 'react'
import { createNewRAGProvider } from '../../../../store'

interface AddRAGProviderDrawerProps {
  open?: boolean
  onClose?: () => void
}

const RAG_PROVIDER_OPTIONS = [
  { value: 'local', label: '🏠 Local' },
  { value: 'lightrag', label: '🔍 LightRAG' },
  { value: 'ragstack', label: '📚 RAGStack' },
  { value: 'chroma', label: '🌈 ChromaDB' },
  { value: 'weaviate', label: '🕷️ Weaviate' },
  { value: 'pinecone', label: '🌲 Pinecone' },
  { value: 'custom', label: '🔧 Custom' },
]

export function AddRAGProviderDrawer({ open = false, onClose }: AddRAGProviderDrawerProps) {
  const [form] = Form.useForm()
  const [loading, setLoading] = useState(false)

  const handleSubmit = async (values: any) => {
    setLoading(true)
    try {
      await createNewRAGProvider({
        name: values.name,
        type: values.type,
        enabled: values.enabled ?? true,
        api_key: values.api_key,
        base_url: values.base_url,
      })
      form.resetFields()
      onClose?.()
    } catch (error) {
      console.error('Failed to create RAG provider:', error)
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
      title="Add RAG Provider"
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
            Add Provider
          </Button>
        </div>
      }
    >
      <Form
        form={form}
        layout="vertical"
        onFinish={handleSubmit}
        initialValues={{
          enabled: true,
        }}
      >
        <Form.Item
          label="Name"
          name="name"
          rules={[{ required: true, message: 'Please enter a name' }]}
        >
          <Input placeholder="Enter provider name" />
        </Form.Item>

        <Form.Item
          label="Type"
          name="type"
          rules={[{ required: true, message: 'Please select a type' }]}
        >
          <Select options={RAG_PROVIDER_OPTIONS} placeholder="Select provider type" />
        </Form.Item>

        <Form.Item
          label="API Key"
          name="api_key"
        >
          <Input.Password placeholder="Enter API key (if required)" />
        </Form.Item>

        <Form.Item
          label="Base URL"
          name="base_url"
        >
          <Input placeholder="Enter base URL (if required)" />
        </Form.Item>
      </Form>
    </Drawer>
  )
}