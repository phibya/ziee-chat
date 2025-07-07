import {useState} from 'react'
import {Layout, Menu, Typography} from 'antd'
import {SettingOutlined, UserOutlined, EyeOutlined, LockOutlined, ToolOutlined, SlidersOutlined, ExperimentOutlined} from '@ant-design/icons'
import {GeneralSettings, AppearanceSettings, PrivacySettings, ModelProvidersSettings} from './Settings'

const {Title} = Typography
const {Sider, Content} = Layout

export function SettingsPage() {
    const [selectedCategory, setSelectedCategory] = useState('general')

    const menuItems = [
        {
            key: 'general',
            icon: <UserOutlined />,
            label: 'General',
        },
        {
            key: 'appearance',
            icon: <EyeOutlined />,
            label: 'Appearance',
        },
        {
            key: 'privacy',
            icon: <LockOutlined />,
            label: 'Privacy',
        },
        {
            key: 'model-providers',
            icon: <ToolOutlined />,
            label: 'Model Providers',
        },
        {
            key: 'shortcuts',
            icon: <SlidersOutlined />,
            label: 'Shortcuts',
        },
        {
            key: 'hardware',
            icon: <ToolOutlined />,
            label: 'Hardware',
        },
        {
            key: 'local-api-server',
            icon: <ToolOutlined />,
            label: 'Local API Server',
        },
        {
            key: 'https-proxy',
            icon: <ToolOutlined />,
            label: 'HTTPS Proxy',
        },
        {
            key: 'extensions',
            icon: <ExperimentOutlined />,
            label: 'Extensions',
        },
    ]

    const renderContent = () => {
        switch (selectedCategory) {
            case 'general':
                return <GeneralSettings />
            case 'appearance':
                return <AppearanceSettings />
            case 'privacy':
                return <PrivacySettings />
            case 'model-providers':
                return <ModelProvidersSettings />
            default:
                return (
                    <div className="space-y-6">
                        <Title level={3}>{selectedCategory.charAt(0).toUpperCase() + selectedCategory.slice(1)}</Title>
                        <div className="text-gray-500">Settings for {selectedCategory} will be implemented here.</div>
                    </div>
                )
        }
    }

    return (
        <div className="h-full">
            <Layout className="h-full">
                <Sider 
                    width={200} 
                    className="bg-gray-50 border-r border-gray-200"
                    theme="light"
                >
                    <div className="p-4 border-b border-gray-200">
                        <Title level={4} className="mb-0 flex items-center">
                            <SettingOutlined className="mr-2" />
                            Settings
                        </Title>
                    </div>
                    <Menu
                        mode="inline"
                        selectedKeys={[selectedCategory]}
                        items={menuItems}
                        className="border-none"
                        onClick={({ key }) => setSelectedCategory(key)}
                    />
                </Sider>
                <Layout>
                    <Content className="p-6 overflow-auto">
                        {renderContent()}
                    </Content>
                </Layout>
            </Layout>
        </div>
    )
}