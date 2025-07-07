import { useState, useCallback, useEffect } from 'react'
import { Button, Layout } from 'antd'
import { CloseOutlined, MenuOutlined } from '@ant-design/icons'
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
  const appTheme = useTheme()
  const { leftPanelCollapsed, leftPanelWidth, setLeftPanelCollapsed, setLeftPanelWidth } = useSettingsStore()
  const [isResizing, setIsResizing] = useState(false)
  
  const MIN_WIDTH = 180
  const MAX_WIDTH = 400
  const COLLAPSE_THRESHOLD = 120

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault()
    setIsResizing(true)
  }, [])

  const handleMouseMove = useCallback(
    (e: MouseEvent) => {
      if (!isResizing) return

      requestAnimationFrame(() => {
        const newWidth = e.clientX
        if (newWidth < COLLAPSE_THRESHOLD) {
          setLeftPanelCollapsed(true)
        } else {
          setLeftPanelCollapsed(false)
          const clampedWidth = Math.min(Math.max(newWidth, MIN_WIDTH), MAX_WIDTH)
          setLeftPanelWidth(clampedWidth)
        }
      })
    },
    [isResizing, setLeftPanelCollapsed, setLeftPanelWidth]
  )

  const handleMouseUp = useCallback(() => {
    setIsResizing(false)
  }, [])

  useEffect(() => {
    if (isResizing) {
      document.addEventListener('mousemove', handleMouseMove)
      document.addEventListener('mouseup', handleMouseUp)
      document.body.style.cursor = 'col-resize'
      document.body.style.userSelect = 'none'

      return () => {
        document.removeEventListener('mousemove', handleMouseMove)
        document.removeEventListener('mouseup', handleMouseUp)
        document.body.style.cursor = ''
        document.body.style.userSelect = ''
      }
    }
  }, [isResizing, handleMouseMove, handleMouseUp])
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
        width={leftPanelCollapsed ? 60 : leftPanelWidth}
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
          transition: isResizing ? 'none' : 'width 0.2s ease',
        }}
        className="hidden lg:block"
      >
        <LeftPanel />
      </Sider>

      {/* Resize Handle */}
      {!leftPanelCollapsed && (
        <div
          className="hidden lg:block fixed top-0 bottom-0 w-1 cursor-col-resize hover:bg-blue-500 transition-colors duration-200 z-[1001]"
          style={{
            left: `${leftPanelWidth - 2}px`,
            backgroundColor: isResizing ? appTheme.primary : 'transparent',
          }}
          onMouseDown={handleMouseDown}
        />
      )}

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
          marginLeft: leftPanelCollapsed ? 60 : leftPanelWidth,
          transition: isResizing ? 'none' : 'margin-left 0.2s',
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
