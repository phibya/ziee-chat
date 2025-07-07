import {Typography, Space, Card} from 'antd'

const {Title, Text} = Typography

export function HardwareSettings() {
    return (
        <Space direction="vertical" size="large" style={{ width: '100%' }}>
            <Title level={3}>Hardware</Title>
            <Card title="Hardware Configuration">
                <Text type="secondary">Hardware configuration settings will be implemented here.</Text>
            </Card>
        </Space>
    )
}