import { Card, Space, Typography } from 'antd'

const { Title, Text } = Typography

export function ModelProvidersSettings() {
  return (
    <Space direction="vertical" size="large" style={{ width: '100%' }}>
      <Title level={3}>Model Providers</Title>
      <Card title="API Configuration">
        <Text type="secondary">
          Model provider settings will be implemented here.
        </Text>
      </Card>
    </Space>
  )
}
