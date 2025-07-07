import { Layout, Menu, Typography } from 'antd'
import { useNavigate, useLocation, Outlet } from 'react-router-dom'
import {
  SettingOutlined,
  UserOutlined,
  EyeOutlined,
  LockOutlined,
  ToolOutlined,
  SlidersOutlined,
  ExperimentOutlined,
} from '@ant-design/icons'

const { Title } = Typography
const { Sider, Content } = Layout

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
    <Layout style={{ height: '100%' }}>
      <Sider width={200} theme="light">
        <div style={{ padding: '16px' }}>
          <Title level={4} style={{ margin: 0 }}>
            <SettingOutlined style={{ marginRight: 8 }} />
            Settings
          </Title>
        </div>
        <Menu
          mode="inline"
          selectedKeys={[currentSection]}
          items={menuItems}
          onClick={({ key }) => handleMenuClick(key)}
        />
      </Sider>
      <Layout>
        <Content style={{ padding: '24px', overflow: 'auto' }}>
          <Outlet />
        </Content>
      </Layout>
    </Layout>
  )
}
