import { MenuUnfoldOutlined } from '@ant-design/icons'
import { Button, Layout } from 'antd'
import { useEffect } from 'react'
import {
  getUILeftPanelCollapsed,
  setIsMobile,
  setMobileOverlayOpen,
  setUILeftPanelCollapsed,
  Stores,
} from '../../store'
import { LeftPanel } from './LeftPanel'

const { Sider, Content } = Layout

interface AppLayoutProps {
  children: React.ReactNode
}

export function AppLayout({ children }: AppLayoutProps) {
  const leftPanelCollapsed = getUILeftPanelCollapsed()
  const { isMobile, mobileOverlayOpen } = Stores.UI.Layout

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
  }, [isMobile, mobileOverlayOpen, setIsMobile, setMobileOverlayOpen])

  // Force collapse on mobile
  useEffect(() => {
    if (isMobile && !leftPanelCollapsed) {
      setUILeftPanelCollapsed(true)
    }
  }, [isMobile, leftPanelCollapsed])

  return (
    <Layout className={'h-screen overflow-hidden'}>
      {/* Left Panel - Only show when not collapsed on desktop or when overlay is open on mobile */}
      {(!isMobile && !leftPanelCollapsed) || (isMobile && mobileOverlayOpen) ? (
        <Sider
          width={'fit-content'}
          collapsible
          collapsed={false}
          trigger={null}
          breakpoint="lg"
          collapsedWidth={0}
          className={`overflow-auto h-screen fixed top-0 left-0 bottom-0 z-1000 ${
            isMobile ? 'z-[1050]' : ''
          }`}
          theme={'light'}
        >
          <LeftPanel />
        </Sider>
      ) : null}

      {/* Floating Toggle Button - Only show when panel is collapsed on desktop */}
      {!isMobile && leftPanelCollapsed && (
        <div className="fixed top-4 left-4 z-[1060]">
          <Button
            type="default"
            icon={<MenuUnfoldOutlined />}
            onClick={() => setUILeftPanelCollapsed(false)}
          />
        </div>
      )}

      {/* Mobile Toggle Button - Show when panel is not open on mobile */}
      {isMobile && !mobileOverlayOpen && (
        <div className="fixed top-4 left-4 z-[1060]">
          <button
            onClick={() => setMobileOverlayOpen(true)}
            className="bg-white border border-gray-300 rounded-md p-2 shadow-md hover:bg-gray-50 transition-colors"
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              width: '40px',
              height: '40px',
            }}
          >
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <path d="M3 12h18M3 6h18M3 18h18" />
            </svg>
          </button>
        </div>
      )}

      {/* Mobile Overlay Backdrop */}
      {isMobile && mobileOverlayOpen && (
        <div
          className="fixed inset-0 z-[1040]"
          style={{ backgroundColor: 'rgba(0, 0, 0, 0.5)' }}
          onClick={() => setMobileOverlayOpen(false)}
        />
      )}

      {/* Main Content */}
      <Content className="w-full h-screen overflow-hidden">{children}</Content>
    </Layout>
  )
}
