import {Typography, Space, Card} from 'antd'

const {Title, Text} = Typography

export function HttpsProxySettings() {
    return (
        <Space direction="vertical" size="large" style={{ width: '100%' }}>
            <Title level={3}>HTTPS Proxy</Title>
            <Card title="Proxy Configuration">
                <Text type="secondary">HTTPS proxy configuration will be implemented here.</Text>
            </Card>
        </Space>
    )
}