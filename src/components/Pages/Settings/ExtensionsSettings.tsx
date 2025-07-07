import {Typography, Space} from 'antd'

const {Title, Text} = Typography

export function ExtensionsSettings() {
    return (
        <Space direction="vertical" size="large" style={{ width: '100%' }}>
            <Title level={3}>Extensions</Title>
            <Text type="secondary">Extensions management will be implemented here.</Text>
        </Space>
    )
}