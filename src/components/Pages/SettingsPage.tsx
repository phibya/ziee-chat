import {useEffect} from 'react'
import {Layout, Menu, Typography} from 'antd'
import {useNavigate, useLocation, Outlet} from 'react-router-dom'
import {SettingOutlined, UserOutlined, EyeOutlined, LockOutlined, ToolOutlined, SlidersOutlined, ExperimentOutlined} from '@ant-design/icons'

const {Title} = Typography
const {Sider, Content} = Layout

export function SettingsPage() {
    const navigate = useNavigate()
    const location = useLocation()

    // Extract the current settings section from the URL
    const currentSection = location.pathname.split('/').pop() || 'general'

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

    const handleMenuClick = (key: string) => {
        navigate(`/settings/${key}`)
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
                        selectedKeys={[currentSection]}
                        items={menuItems}
                        className="border-none"
                        onClick={({ key }) => handleMenuClick(key)}
                    />
                </Sider>
                <Layout>
                    <Content className="p-6 overflow-auto">
                        <Outlet />
                    </Content>
                </Layout>
            </Layout>
        </div>
    )
}