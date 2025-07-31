import { Button, Card, Form, Input, Typography } from 'antd'
import { useState } from 'react'
import { RAGProvider } from '../../../../../types/api/ragProvider'
import { updateRAGProvider } from '../../../../../store'

const { Title } = Typography

interface RAGConfigurationSectionProps {
  provider: RAGProvider
}

export function RAGConfigurationSection({ provider }: RAGConfigurationSectionProps) {
  const [form] = Form.useForm()
  const [loading, setLoading] = useState(false)

  const handleSave = async (values: any) => {
    setLoading(true)
    try {
      await updateRAGProvider(provider.id, {
        api_key: values.api_key,
        base_url: values.base_url,
      })
    } catch (error) {
      console.error('Failed to update RAG provider configuration:', error)
    } finally {
      setLoading(false)
    }
  }

  return (
    <Card style={{ marginBottom: 24 }}>
      <Title level={4}>Configuration</Title>
      <Form
        form={form}
        layout="vertical"
        onFinish={handleSave}
        initialValues={{
          api_key: provider.api_key,
          base_url: provider.base_url,
        }}
      >
        <Form.Item
          label="API Key"
          name="api_key"
        >
          <Input.Password placeholder="Enter API key" />
        </Form.Item>
        
        <Form.Item
          label="Base URL"
          name="base_url"
        >
          <Input placeholder="Enter base URL" />
        </Form.Item>

        <Form.Item>
          <Button type="primary" htmlType="submit" loading={loading}>
            Save Configuration
          </Button>
        </Form.Item>
      </Form>
    </Card>
  )
}