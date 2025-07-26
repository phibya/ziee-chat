import {
  CloudDownloadOutlined,
  ExperimentOutlined,
  EyeOutlined,
  LockOutlined,
  MenuOutlined,
  RobotOutlined,
  SettingOutlined,
  SlidersOutlined,
  TeamOutlined,
  ToolOutlined,
  UserOutlined,
} from '@ant-design/icons'
import { Button, Drawer, Layout, Menu, theme, Typography } from 'antd'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Outlet, useLocation, useNavigate } from 'react-router-dom'
import { isDesktopApp } from '../../api/core'
import { Permission, usePermissions } from '../../permissions'
import { PageContainer } from '../common/PageContainer.tsx'

const { Title } = Typography
const { Sider } = Layout

export function SettingsPage() {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const location = useLocation()
  const [isMobile, setIsMobile] = useState(false)
  const [drawerVisible, setDrawerVisible] = useState(false)
  const { hasPermission } = usePermissions()
  const { token } = theme.useToken()

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
      label: t('settings.general'),
    },
    {
      key: 'appearance',
      icon: <EyeOutlined />,
      label: t('settings.appearance'),
    },
    {
      key: 'privacy',
      icon: <LockOutlined />,
      label: t('settings.privacy'),
    },
    // Providers only shows in main menu for desktop apps
    ...(isDesktopApp
      ? [
          {
            key: 'providers',
            icon: <ToolOutlined />,
            label: t('settings.providers'),
          },
          {
            key: 'repositories',
            icon: <CloudDownloadOutlined />,
            label: t('settings.modelRepository.title'),
          },
        ]
      : []),

    {
      key: 'shortcuts',
      icon: <SlidersOutlined />,
      label: t('settings.shortcuts'),
    },
    {
      key: 'hardware',
      icon: <ToolOutlined />,
      label: t('settings.hardware'),
    },
    // HTTPS Proxy only shows in main menu for desktop apps
    ...(isDesktopApp
      ? [
          {
            key: 'https-proxy',
            icon: <ToolOutlined />,
            label: t('settings.httpsProxy'),
          },
        ]
      : []),
    {
      key: 'extensions',
      icon: <ExperimentOutlined />,
      label: t('settings.extensions'),
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
        const hasProviderManagement = hasPermission(
          Permission.config.providers.read,
        )
        const hasRepositoryManagement = hasPermission(
          Permission.config.repositories.read,
        )
        const hasProxyManagement = hasPermission(Permission.config.proxy.read)
        const hasAssistantsManagement = hasPermission(
          Permission.config.assistants.read,
        )

        if (
          hasUserManagement ||
          hasGroupManagement ||
          hasAppearanceManagement ||
          hasProviderManagement ||
          hasRepositoryManagement ||
          hasProxyManagement ||
          hasAssistantsManagement
        ) {
          items.push({
            type: 'divider' as const,
          })
          items.push({
            key: 'admin',
            icon: <SettingOutlined />,
            label: t('settings.admin'),
            type: 'group' as const,
          })

          if (hasAppearanceManagement) {
            items.push({
              key: 'admin-general',
              icon: <UserOutlined />,
              label: t('settings.general'),
            })
          }

          if (hasAppearanceManagement) {
            items.push({
              key: 'admin-appearance',
              icon: <EyeOutlined />,
              label: t('settings.appearance'),
            })
          }

          if (hasProviderManagement) {
            items.push({
              key: 'providers',
              icon: <ToolOutlined />,
              label: t('settings.providers'),
            })
          }

          if (hasRepositoryManagement) {
            items.push({
              key: 'repositories',
              icon: <CloudDownloadOutlined />,
              label: t('settings.modelRepository.title'),
            })
          }

          if (hasProxyManagement) {
            items.push({
              key: 'https-proxy',
              icon: <ToolOutlined />,
              label: t('settings.httpsProxy'),
            })
          }

          if (hasAssistantsManagement) {
            items.push({
              key: 'admin-assistants',
              icon: <RobotOutlined />,
              label: t('settings.assistants'),
            })
          }

          if (hasUserManagement) {
            items.push({
              key: 'users',
              icon: <UserOutlined />,
              label: t('settings.users'),
            })
          }

          if (hasGroupManagement) {
            items.push({
              key: 'user-groups',
              icon: <TeamOutlined />,
              label: t('settings.userGroups'),
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
      <div className={'p-2.5'}>
        <Title level={4} style={{ margin: 0 }}>
          <SettingOutlined style={{ marginRight: 8 }} />
          {t('settings.title')}
        </Title>
      </div>
      <Menu
        className={'w-fit'}
        style={{
          lineHeight: 1,
        }}
        selectedKeys={[currentSection]}
        items={menuItems}
        onClick={({ key }) => handleMenuClick(key)}
      />
    </>
  )

  return (
    <Layout className="h-screen w-full">
      {/* Mobile Header */}
      {isMobile && (
        <div className="flex items-center justify-between p-4">
          <Button
            type="text"
            icon={<MenuOutlined />}
            onClick={() => setDrawerVisible(true)}
          />
          <Title level={3} style={{ margin: 0 }}>
            <SettingOutlined style={{ marginRight: 8 }} />
            {t('settings.title')}
          </Title>
          <div className="w-8" />
        </div>
      )}

      {/* Desktop Sidebar */}
      {!isMobile && (
        <Sider
          theme={'light'}
          className={'h-screen overflow-auto w-fit px-1'}
          width={'fit-content'}
          style={{
            borderRight: `1px solid ${token.colorBorderSecondary}`,
          }}
        >
          <SettingsMenu />
        </Sider>
      )}

      {/* Mobile Drawer */}
      <Drawer
        title={null}
        placement="left"
        onClose={() => setDrawerVisible(false)}
        open={drawerVisible}
        styles={{
          body: { padding: 0 },
        }}
        width={280}
      >
        <SettingsMenu />
      </Drawer>

      {/* Main Content */}
      <Layout>
        <PageContainer>
          <Outlet />
        </PageContainer>
      </Layout>
    </Layout>
  )
}
