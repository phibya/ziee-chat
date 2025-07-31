import { App, Button, Drawer, Form, Input, InputNumber, Switch } from 'antd'
import {
  addNewDatabaseToRAGProvider,
  useAddRAGDatabaseDrawerStore,
} from '../../../../store'

interface AddRAGDatabaseDrawerProps {
  open?: boolean
  onClose?: () => void
  providerId?: string
}

export function AddRAGDatabaseDrawer({
  open = false,
  onClose,
  providerId,
}: AddRAGDatabaseDrawerProps) {
  const [form] = Form.useForm()
  const { message } = App.useApp()
  const { loading } = useAddRAGDatabaseDrawerStore()

  const handleSubmit = async (values: any) => {
    if (!providerId) {
      message.error('No provider selected')
      return
    }

    try {
      await addNewDatabaseToRAGProvider(providerId, {
        name: values.name,
        alias: values.alias,
        description: values.description,
        collection_name: values.collection_name,
        embedding_model: values.embedding_model,
        chunk_size: values.chunk_size,
        chunk_overlap: values.chunk_overlap,
        enabled: values.enabled ?? true,
      })

      message.success('RAG database created successfully')
      form.resetFields()
      onClose?.()
    } catch (error) {
      console.error('Failed to create RAG database:', error)
      message.error('Failed to create RAG database')
    }
  }

  const handleClose = () => {
    form.resetFields()
    onClose?.()
  }

  return (
    <Drawer
      title="Add RAG Database"
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
            Add Database
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
          chunk_size: 1000,
          chunk_overlap: 200,
        }}
      >
        <Form.Item
          label="Name"
          name="name"
          rules={[{ required: true, message: 'Please enter a name' }]}
        >
          <Input placeholder="Enter database name" />
        </Form.Item>

        <Form.Item
          label="Alias"
          name="alias"
          rules={[{ required: true, message: 'Please enter an alias' }]}
        >
          <Input placeholder="Enter database alias" />
        </Form.Item>

        <Form.Item label="Description" name="description">
          <Input.TextArea placeholder="Enter description" rows={3} />
        </Form.Item>

        <Form.Item label="Collection Name" name="collection_name">
          <Input placeholder="Enter collection name" />
        </Form.Item>

        <Form.Item label="Embedding Model" name="embedding_model">
          <Input placeholder="Enter embedding model" />
        </Form.Item>

        <Form.Item label="Chunk Size" name="chunk_size">
          <InputNumber min={100} max={10000} style={{ width: '100%' }} />
        </Form.Item>

        <Form.Item label="Chunk Overlap" name="chunk_overlap">
          <InputNumber min={0} max={1000} style={{ width: '100%' }} />
        </Form.Item>

        <Form.Item label="Enabled" name="enabled" valuePropName="checked">
          <Switch />
        </Form.Item>
      </Form>
    </Drawer>
  )
}
