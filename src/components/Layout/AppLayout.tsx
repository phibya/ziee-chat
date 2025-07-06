import { useState } from 'react'
import { Layout, Button, theme } from 'antd'
import { MenuOutlined, CloseOutlined } from '@ant-design/icons'
import { useTranslation } from 'react-i18next'
import { useSettingsStore } from '../../store/settings'
import { LeftPanel } from './LeftPanel'

const { Sider, Content } = Layout

interface AppLayoutProps {
  children: React.ReactNode
}

export function AppLayout({ children }: AppLayoutProps) {
  const { t } = useTranslation()
  const { token } = theme.useToken()
  const { leftPanelCollapsed, setLeftPanelCollapsed } = useSettingsStore()
  const [mobileDrawerOpen, setMobileDrawerOpen] = useState(false)

  return (
    <Layout style={{ height: '100vh', overflow: 'hidden' }}>
      {/* Mobile Header */}
      <div 
        className="lg:hidden flex items-center justify-between p-4 border-b"
        style={{ borderColor: token.colorBorderSecondary }}
      >
        <Button
          type="text"
          icon={<MenuOutlined />}
          onClick={() => setMobileDrawerOpen(true)}
        />
        <span style={{ color: token.colorText, fontWeight: 600 }}>
          {t('app.title')}
        </span>
        <div style={{ width: 32 }} />
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
          backgroundColor: 'rgb(20, 20, 20)',
          borderRight: `1px solid rgba(255,255,255,0.1)`,
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
            className="lg:hidden fixed inset-0 bg-black bg-opacity-50 z-40"
            onClick={() => setMobileDrawerOpen(false)}
          />
          <div 
            className="lg:hidden fixed left-0 top-0 bottom-0 w-80 max-w-[80vw] z-50 transform transition-transform duration-300 ease-in-out"
            style={{ 
              backgroundColor: token.colorBgContainer,
              transform: mobileDrawerOpen ? 'translateX(0)' : 'translateX(-100%)',
            }}
          >
            <div className="flex items-center justify-between p-4 border-b">
              <span style={{ color: token.colorText, fontWeight: 600 }}>
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
        className="lg:ml-0"
      >
        <Content
          style={{
            margin: 0,
            padding: 0,
            minHeight: 280,
            backgroundColor: 'rgb(26, 26, 26)',
            display: 'flex',
            flexDirection: 'column',
          }}
        >
          {children}
        </Content>
      </Layout>
    </Layout>
  )
}