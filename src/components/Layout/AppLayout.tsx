import { useState } from 'react'
import { Layout, Button, theme } from 'antd'
import { MenuOutlined, CloseOutlined } from '@ant-design/icons'
import { useTranslation } from 'react-i18next'
import { useSettingsStore } from '../../store/settings'
import { LeftPanel } from './LeftPanel'
import { useTheme } from '../../hooks/useTheme'

const { Sider, Content } = Layout

interface AppLayoutProps {
  children: React.ReactNode
}

export function AppLayout({ children }: AppLayoutProps) {
  const { t } = useTranslation()
  const { token } = theme.useToken()
  const appTheme = useTheme()
  const { leftPanelCollapsed } = useSettingsStore()
  const [mobileDrawerOpen, setMobileDrawerOpen] = useState(false)

  return (
    <Layout style={{ height: '100vh', overflow: 'hidden' }}>
      {/* Mobile Header */}
      <div
        className="lg:hidden flex items-center justify-between p-4 border-b"
        style={{ borderColor: appTheme.borderSecondary }}
      >
        <Button
          type="text"
          icon={<MenuOutlined />}
          onClick={() => setMobileDrawerOpen(true)}
        />
        <span className="font-semibold" style={{ color: appTheme.textPrimary }}>
          {t('app.title')}
        </span>
        <div className="w-8" />
      </div>

      {/* Desktop Sidebar */}
      <Sider
        width={leftPanelCollapsed ? 60 : 280}
        collapsible
        collapsed={false}
        trigger={null}
        breakpoint="lg"
        collapsedWidth={60}
        style={{
          overflow: 'auto',
          height: '100vh',
          position: 'fixed',
          left: 0,
          top: 0,
          bottom: 0,
          zIndex: 1000,
          backgroundColor: appTheme.sidebarBackground,
          borderRight: `1px solid ${appTheme.sidebarBorder}`,
          transition: 'width 0.2s ease',
        }}
        className="hidden lg:block"
      >
        <LeftPanel />
      </Sider>

      {/* Mobile Drawer */}
      {mobileDrawerOpen && (
        <>
          <div
            className="lg:hidden fixed inset-0 z-40"
            style={{ backgroundColor: 'rgba(0, 0, 0, 0.5)' }}
            onClick={() => setMobileDrawerOpen(false)}
          />
          <div
            className="lg:hidden fixed left-0 top-0 bottom-0 w-80 max-w-[80vw] z-50 transform transition-transform duration-300 ease-in-out"
            style={{
              backgroundColor: appTheme.surface,
              transform: mobileDrawerOpen
                ? 'translateX(0)'
                : 'translateX(-100%)',
            }}
          >
            <div
              className="flex items-center justify-between p-4 border-b"
              style={{ borderColor: appTheme.borderSecondary }}
            >
              <span
                className="font-semibold"
                style={{ color: appTheme.textPrimary }}
              >
                {t('app.title')}
              </span>
              <Button
                type="text"
                icon={<CloseOutlined />}
                onClick={() => setMobileDrawerOpen(false)}
              />
            </div>
            <LeftPanel onItemClick={() => setMobileDrawerOpen(false)} />
          </div>
        </>
      )}

      {/* Main Content */}
      <Layout
        style={{
          marginLeft: leftPanelCollapsed ? 60 : 280,
          transition: 'margin-left 0.2s',
        }}
        className="lg:ml-0 transition-all duration-200"
      >
        <Content
          className="m-0 p-0 min-h-72 flex flex-col"
          style={{ backgroundColor: appTheme.background }}
        >
          {children}
        </Content>
      </Layout>
    </Layout>
  )
}
