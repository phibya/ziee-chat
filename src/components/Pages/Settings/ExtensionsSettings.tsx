import {Typography} from 'antd'

const {Title, Text} = Typography

export function ExtensionsSettings() {
    return (
        <div className="space-y-6">
            <div className="mb-6">
                <Title level={3} className="mb-0">Extensions</Title>
            </div>
            
            <div className="border-b pb-4">
                <Text>Extensions management will be implemented here.</Text>
            </div>
        </div>
    )
}