import { useCallback, useEffect, useState } from 'react'
import { Layout } from 'antd'
import { useSettingsStore } from '../../store/settings'
import { LeftPanel } from './LeftPanel'
import { useTheme } from '../../hooks/useTheme'

const { Sider, Content } = Layout

interface AppLayoutProps {
  children: React.ReactNode
}

export function AppLayout({ children }: AppLayoutProps) {
  const appTheme = useTheme()
  const {
    leftPanelCollapsed,
    leftPanelWidth,
    setLeftPanelCollapsed,
    setLeftPanelWidth,
  } = useSettingsStore()
  const [isResizing, setIsResizing] = useState(false)
  const [isMobile, setIsMobile] = useState(false)
  const [mobileOverlayOpen, setMobileOverlayOpen] = useState(false)

  const MIN_WIDTH = 180
  const MAX_WIDTH = 400
  const COLLAPSE_THRESHOLD = 120

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault()
    setIsResizing(true)
  }, [])

  const handleMouseMove = useCallback(
    // eslint-disable-next-line no-undef
    (e: MouseEvent) => {
      if (!isResizing) return

      // eslint-disable-next-line no-undef
      requestAnimationFrame(() => {
        const newWidth = e.clientX
        if (newWidth < COLLAPSE_THRESHOLD) {
          setLeftPanelCollapsed(true)
        } else {
          setLeftPanelCollapsed(false)
          const clampedWidth = Math.min(
            Math.max(newWidth, MIN_WIDTH),
            MAX_WIDTH,
          )
          setLeftPanelWidth(clampedWidth)
        }
      })
    },
    [isResizing, setLeftPanelCollapsed, setLeftPanelWidth],
  )

  const handleMouseUp = useCallback(() => {
    setIsResizing(false)
  }, [])

  // Check if screen is mobile size
  useEffect(() => {
    const checkMobile = () => {
      const wasMobile = isMobile
      const isNowMobile = window.innerWidth < 1024 // lg breakpoint
      setIsMobile(isNowMobile)

      // Close mobile overlay when switching from mobile to desktop
      if (wasMobile && !isNowMobile && mobileOverlayOpen) {
        setMobileOverlayOpen(false)
      }
    }

    checkMobile()
    window.addEventListener('resize', checkMobile)

    return () => window.removeEventListener('resize', checkMobile)
  }, [isMobile, mobileOverlayOpen])

  // Force collapse on mobile
  useEffect(() => {
    if (isMobile && !leftPanelCollapsed) {
      setLeftPanelCollapsed(true)
    }
  }, [isMobile, leftPanelCollapsed, setLeftPanelCollapsed])

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

  return (
    <Layout style={{ height: '100vh', overflow: 'hidden' }}>
      {/* Desktop Sidebar */}
      <Sider
        width={
          isMobile
            ? mobileOverlayOpen
              ? leftPanelWidth
              : 60
            : leftPanelCollapsed
              ? 60
              : leftPanelWidth
        }
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
          zIndex: isMobile && mobileOverlayOpen ? 1050 : 1000,
          backgroundColor: appTheme.sidebarBackground,
          borderRight: `1px solid ${appTheme.sidebarBorder}`,
          transition: isResizing ? 'none' : 'width 0.2s ease',
          boxShadow:
            isMobile && mobileOverlayOpen
              ? '0 4px 12px rgba(0, 0, 0, 0.15)'
              : 'none',
        }}
        className="block"
      >
        <LeftPanel
          onItemClick={() => {
            if (isMobile) {
              setMobileOverlayOpen(false)
            }
          }}
          isMobile={isMobile}
          mobileOverlayOpen={mobileOverlayOpen}
          setMobileOverlayOpen={setMobileOverlayOpen}
        />
      </Sider>

      {/* Mobile Overlay Backdrop */}
      {isMobile && mobileOverlayOpen && (
        <div
          className="fixed inset-0 bg-black bg-opacity-50 z-[1040]"
          onClick={() => setMobileOverlayOpen(false)}
        />
      )}

      {/* Resize Handle */}
      {!leftPanelCollapsed && !isMobile && (
        <div
          className="fixed top-0 bottom-0 w-1 cursor-col-resize hover:bg-blue-500 transition-colors duration-200 z-[1001]"
          style={{
            left: `${leftPanelWidth - 2}px`,
            backgroundColor: isResizing ? appTheme.primary : 'transparent',
          }}
          onMouseDown={handleMouseDown}
        />
      )}

      {/* Main Content */}
      <Layout
        style={{
          marginLeft: isMobile
            ? mobileOverlayOpen
              ? 0
              : 60
            : leftPanelCollapsed
              ? 60
              : leftPanelWidth,
          transition: isResizing ? 'none' : 'margin-left 0.2s',
        }}
        className="transition-all duration-200"
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
