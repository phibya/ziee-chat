import { useState } from 'react'
import {
  Avatar,
  Button,
  Card,
  Form,
  Input,
  List,
  Modal,
  Typography,
} from 'antd'
import {
  DeleteOutlined,
  EditOutlined,
  PlusOutlined,
  UserOutlined,
} from '@ant-design/icons'
import { useAppStore } from '../../store'

const { Title, Text } = Typography
const { TextArea } = Input

export function AssistantsPage() {
  const { assistants, createAssistant, updateAssistant, deleteAssistant } =
    useAppStore()
  const [isModalVisible, setIsModalVisible] = useState(false)
  const [editingAssistant, setEditingAssistant] = useState<any>(null)
  const [form] = Form.useForm()

  const handleCreateAssistant = () => {
    setEditingAssistant(null)
    form.resetFields()
    setIsModalVisible(true)
  }

  const handleEditAssistant = (assistant: any) => {
    setEditingAssistant(assistant)
    form.setFieldsValue(assistant)
    setIsModalVisible(true)
  }

  const handleDeleteAssistant = (assistantId: string) => {
    Modal.confirm({
      title: 'Delete Assistant',
      content: 'Are you sure you want to delete this assistant?',
      onOk: () => {
        deleteAssistant(assistantId)
      },
    })
  }

  const handleModalOk = () => {
    form.validateFields().then(values => {
      if (editingAssistant) {
        updateAssistant(editingAssistant.id, values)
      } else {
        createAssistant(values)
      }
      setIsModalVisible(false)
      form.resetFields()
    })
  }

  const handleModalCancel = () => {
    setIsModalVisible(false)
    form.resetFields()
  }

  return (
    <div style={{ padding: '24px', height: '100%', overflow: 'auto' }}>
      <div
        style={{
          marginBottom: '24px',
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
        }}
      >
        <Title level={2}>Assistants</Title>
        <Button
          type="primary"
          icon={<PlusOutlined />}
          onClick={handleCreateAssistant}
        >
          Create Assistant
        </Button>
      </div>

      <List
        grid={{ gutter: 16, xs: 1, sm: 2, md: 3, lg: 3, xl: 4 }}
        dataSource={assistants}
        renderItem={assistant => (
          <List.Item>
            <Card
              actions={[
                <EditOutlined
                  key="edit"
                  onClick={() => handleEditAssistant(assistant)}
                />,
                <DeleteOutlined
                  key="delete"
                  onClick={() => handleDeleteAssistant(assistant.id)}
                />,
              ]}
              hoverable
            >
              <Card.Meta
                avatar={<Avatar size="large" icon={<UserOutlined />} />}
                title={assistant.name}
                description={
                  <div>
                    <Text
                      type="secondary"
                      style={{ marginBottom: '8px', display: 'block' }}
                    >
                      {assistant.description}
                    </Text>
                    <Text code style={{ fontSize: '12px' }}>
                      {assistant.model}
                    </Text>
                  </div>
                }
              />
            </Card>
          </List.Item>
        )}
      />

      <Modal
        title={editingAssistant ? 'Edit Assistant' : 'Create Assistant'}
        open={isModalVisible}
        onOk={handleModalOk}
        onCancel={handleModalCancel}
        width={600}
      >
        <Form form={form} layout="vertical" style={{ marginTop: '16px' }}>
          <Form.Item
            label="Name"
            name="name"
            rules={[
              { required: true, message: 'Please input the assistant name!' },
            ]}
          >
            <Input placeholder="Enter assistant name" />
          </Form.Item>

          <Form.Item
            label="Description"
            name="description"
            rules={[
              {
                required: true,
                message: 'Please input the assistant description!',
              },
            ]}
          >
            <TextArea placeholder="Enter assistant description" rows={3} />
          </Form.Item>

          <Form.Item
            label="Model"
            name="model"
            rules={[
              { required: true, message: 'Please input the model name!' },
            ]}
          >
            <Input placeholder="e.g., gpt-3.5-turbo" />
          </Form.Item>

          <Form.Item label="System Prompt" name="systemPrompt">
            <TextArea placeholder="Enter system prompt (optional)" rows={4} />
          </Form.Item>
        </Form>
      </Modal>
    </div>
  )
}
