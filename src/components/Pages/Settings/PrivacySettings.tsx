import { Typography, Space, Card } from 'antd'

const { Title, Text } = Typography

export function PrivacySettings() {
  return (
    <Space direction="vertical" size="large" style={{ width: '100%' }}>
      <Title level={3}>Privacy</Title>
      <Card title="Privacy Controls">
        <Text type="secondary">Privacy settings will be implemented here.</Text>
      </Card>
    </Space>
  )
}
