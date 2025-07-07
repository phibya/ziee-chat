import {Typography, Space} from 'antd'

const {Title, Text} = Typography

export function ShortcutsSettings() {
    return (
        <Space direction="vertical" size="large" style={{ width: '100%' }}>
            <Title level={3}>Shortcuts</Title>
            <Text type="secondary">Keyboard shortcuts settings will be implemented here.</Text>
        </Space>
    )
}