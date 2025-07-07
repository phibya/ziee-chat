import {Typography, Space} from 'antd'

const {Title, Text} = Typography

export function LocalApiServerSettings() {
    return (
        <Space direction="vertical" size="large" style={{ width: '100%' }}>
            <Title level={3}>Local API Server</Title>
            <Text type="secondary">Local API server configuration will be implemented here.</Text>
        </Space>
    )
}