import {Typography} from 'antd'

const {Title, Text} = Typography

export function ModelProvidersSettings() {
    return (
        <div className="space-y-6">
            <div className="mb-6">
                <Title level={3} className="mb-0">Model Providers</Title>
            </div>
            
            <div className="border-b pb-4">
                <Text>Model provider settings will be implemented here.</Text>
            </div>
        </div>
    )
}