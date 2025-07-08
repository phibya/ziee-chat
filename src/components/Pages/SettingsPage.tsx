import { Button, Drawer, Layout, Menu, Typography } from 'antd'
import { Outlet, useLocation, useNavigate } from 'react-router-dom'
import { useEffect, useState } from 'react'
import {
  ExperimentOutlined,
  EyeOutlined,
  LockOutlined,
  MenuOutlined,
  SettingOutlined,
  SlidersOutlined,
  TeamOutlined,
  ToolOutlined,
  UserOutlined,
} from '@ant-design/icons'
import { isDesktopApp } from '../../api/core'
import { Permission, usePermissions } from '../../permissions'

const { Title } = Typography
const { Sider, Content } = Layout

export function SettingsPage() {
  const navigate = useNavigate()
  const location = useLocation()
  const [isMobile, setIsMobile] = useState(false)
  const [drawerVisible, setDrawerVisible] = useState(false)
  const { hasPermission } = usePermissions()

  // Extract the current settings section from the URL
  const currentSection = location.pathname.split('/').pop() || 'general'

  // Check if screen is mobile size
  useEffect(() => {
    const checkMobile = () => {
      setIsMobile(window.innerWidth < 768) // md breakpoint
    }

    checkMobile()
    window.addEventListener('resize', checkMobile)

    return () => window.removeEventListener('resize', checkMobile)
  }, [])

  const baseMenuItems = [
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

  // Build admin menu items based on permissions
  const adminMenuItems = !isDesktopApp
    ? (() => {
        const items = []

        // Check if user has any admin permissions
        const hasUserManagement = hasPermission(Permission.users.read)
        const hasGroupManagement = hasPermission(Permission.groups.read)
        const hasAppearanceManagement = hasPermission(
          Permission.config.experimental.edit,
        )

        if (
          hasUserManagement ||
          hasGroupManagement ||
          hasAppearanceManagement
        ) {
          items.push({
            type: 'divider' as const,
          })
          items.push({
            key: 'admin',
            icon: <SettingOutlined />,
            label: 'Admin',
            type: 'group' as const,
          })

          if (hasAppearanceManagement) {
            items.push({
              key: 'admin-appearance',
              icon: <EyeOutlined />,
              label: 'Appearance',
            })
          }

          if (hasUserManagement) {
            items.push({
              key: 'users',
              icon: <UserOutlined />,
              label: 'Users',
            })
          }

          if (hasGroupManagement) {
            items.push({
              key: 'user-groups',
              icon: <TeamOutlined />,
              label: 'User Groups',
            })
          }
        }

        return items
      })()
    : []

  const menuItems = [...baseMenuItems, ...adminMenuItems]

  const handleMenuClick = (key: string) => {
    navigate(`/settings/${key}`)
    if (isMobile) {
      setDrawerVisible(false)
    }
  }

  const SettingsMenu = () => (
    <>
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
    </>
  )

  return (
    <Layout style={{ height: '100%' }}>
      {/* Mobile Header */}
      {isMobile && (
        <div
          className="flex items-center justify-between p-4"
          style={{ borderColor: '#f0f0f0' }}
        >
          <Button
            type="text"
            icon={<MenuOutlined />}
            onClick={() => setDrawerVisible(true)}
          />
          <Title level={4} style={{ margin: 0 }}>
            <SettingOutlined style={{ marginRight: 8 }} />
            Settings
          </Title>
          <div className="w-8" />
        </div>
      )}

      {/* Desktop Sidebar */}
      {!isMobile && (
        <Sider width={200} theme="light">
          <SettingsMenu />
        </Sider>
      )}

      {/* Mobile Drawer */}
      <Drawer
        title={null}
        placement="left"
        onClose={() => setDrawerVisible(false)}
        open={drawerVisible}
        bodyStyle={{ padding: 0 }}
        width={280}
      >
        <SettingsMenu />
      </Drawer>

      {/* Main Content */}
      <Layout>
        <Content
          style={{
            padding: isMobile ? '16px' : '24px',
            overflow: 'auto',
            marginTop: isMobile ? 0 : 0,
          }}
        >
          <Outlet />
        </Content>
      </Layout>
    </Layout>
  )
}
