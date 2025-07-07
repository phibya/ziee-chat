import {Typography} from 'antd'

const {Title, Text} = Typography

export function HttpsProxySettings() {
    return (
        <div className="space-y-6">
            <div className="mb-6">
                <Title level={3} className="mb-0">HTTPS Proxy</Title>
            </div>
            
            <div className="border-b pb-4">
                <Text>HTTPS proxy configuration will be implemented here.</Text>
            </div>
        </div>
    )
}