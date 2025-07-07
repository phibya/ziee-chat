import {Typography} from 'antd'

const {Title, Text} = Typography

export function LocalApiServerSettings() {
    return (
        <div className="space-y-6">
            <div className="mb-6">
                <Title level={3} className="mb-0">Local API Server</Title>
            </div>
            
            <div className="border-b pb-4">
                <Text>Local API server configuration will be implemented here.</Text>
            </div>
        </div>
    )
}