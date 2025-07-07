import {Typography} from 'antd'

const {Title, Text} = Typography

export function PrivacySettings() {
    return (
        <div className="space-y-6">
            <div className="mb-6">
                <Title level={3} className="mb-0">Privacy</Title>
            </div>
            
            <div className="border-b pb-4">
                <Text>Privacy settings will be implemented here.</Text>
            </div>
        </div>
    )
}