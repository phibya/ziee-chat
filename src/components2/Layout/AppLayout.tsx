import React, { useCallback, useEffect, useRef } from 'react'
import { LeftSidebar } from './LeftSidebar'
import { Stores, setSidebarCollapsed } from '../../store'
import { Button, theme } from 'antd'
import { useWindowMinSize } from '../hooks/useWindowMinSize.ts'
import { isDesktopApp } from '../../api/core.ts'
import { GoSidebarCollapse, GoSidebarExpand } from 'react-icons/go'

interface AppLayoutProps {
  children: React.ReactNode
}

export function AppLayout({ children }: AppLayoutProps) {
  const { isSidebarCollapsed } = Stores.UI.Layout
  const { token } = theme.useToken()
  const windowMinSize = useWindowMinSize()

  const sidebarRef = useRef<HTMLDivElement>(null)
  const spacerRef = useRef<HTMLDivElement>(null)
  const currentWidth = useRef(200)

  const MIN_WIDTH = 150
  const MAX_WIDTH = 400

  const handleMouseDown = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault()

      const handleMouseMove = (e: MouseEvent) => {
        const newWidth = e.clientX

        if (spacerRef.current) {
          spacerRef.current.style.transition = 'none'
        }

        if (newWidth < MIN_WIDTH / 2) {
          if (spacerRef.current) {
            spacerRef.current.style.transition = 'all 200ms ease-out'
          }
          setSidebarCollapsed(true)
        } else if (newWidth >= MIN_WIDTH && newWidth <= MAX_WIDTH) {
          // If coming from collapsed state, re-enable transition for smooth expand
          if (isSidebarCollapsed) {
            if (spacerRef.current) {
              spacerRef.current.style.transition = 'all 200ms ease-out'
            }

            setTimeout(() => {
              setSidebarCollapsed(false)
              currentWidth.current = newWidth
              if (sidebarRef.current) {
                sidebarRef.current.style.width = `${newWidth}px`
              }
              if (spacerRef.current) {
                spacerRef.current.style.width = `${newWidth}px`
              }

              // Resume no-transition dragging after expand
              setTimeout(() => {
                if (sidebarRef.current) {
                  sidebarRef.current.style.transition = 'none'
                }
                if (spacerRef.current) {
                  spacerRef.current.style.transition = 'none'
                }
              }, 300) // Wait for transition to complete
            }, 10)
          } else {
            // Disable the transition for smooth dragging
            setSidebarCollapsed(false)
            currentWidth.current = newWidth
            if (sidebarRef.current) {
              sidebarRef.current.style.width = `${newWidth}px`
            }
            if (spacerRef.current) {
              spacerRef.current.style.width = `${newWidth}px`
            }
          }
        } else if (newWidth > MAX_WIDTH) {
          currentWidth.current = MAX_WIDTH
          if (sidebarRef.current) {
            sidebarRef.current.style.width = `${MAX_WIDTH}px`
          }
          if (spacerRef.current) {
            spacerRef.current.style.width = `${MAX_WIDTH}px`
          }
        }
      }

      const handleMouseUp = () => {
        if (spacerRef.current) {
          spacerRef.current.style.transition = 'all 200ms ease-out'
        }

        document.removeEventListener('mousemove', handleMouseMove)
        document.removeEventListener('mouseup', handleMouseUp)
      }

      document.addEventListener('mousemove', handleMouseMove)
      document.addEventListener('mouseup', handleMouseUp)
    },
    [MIN_WIDTH, MAX_WIDTH],
  )

  useEffect(() => {
    if (windowMinSize.xs) {
      if (!isSidebarCollapsed) {
        setSidebarCollapsed(true)
      }
    }
  }, [windowMinSize.xs])

  const toggleSidebar = () => {
    if (sidebarRef.current) {
      sidebarRef.current.style.transition = 'transform 200ms ease-out'
    }
    setSidebarCollapsed(!isSidebarCollapsed)
    setTimeout(() => {
      if (sidebarRef.current) {
        sidebarRef.current.style.transition = 'none'
      }
    }, 200)
  }

  return (
    <div className="h-screen w-screen flex overflow-hidden ">
      {/* Sidebar - Always visible, width controlled by container */}
      <div
        ref={sidebarRef}
        className="absolute h-full z-1"
        style={{
          width: `${currentWidth.current}px`,
          ...(windowMinSize.xs
            ? {
                zIndex: 3,
                position: 'fixed',
                background: token.colorBgContainer,
                borderRight: '1px solid ' + token.colorBorder,
                transform: isSidebarCollapsed
                  ? 'translateX(-100%)'
                  : 'translateX(0)',
                width: '100%',
              }
            : {}),
        }}
      >
        <LeftSidebar />
      </div>

      <div
        className="flex items-center gap-6 mr-4 fixed z-10 h-[50px]"
        style={{
          left: isDesktopApp ? 86 : 12,
          top: 0,
        }}
      >
        {/* Collapse/Expand Sidebar Button */}
        <Button
          type="text"
          onClick={toggleSidebar}
          className="flex items-center justify-center"
          style={{
            width: '24px',
            height: '24px',
            padding: 0,
            fontSize: '30px',
            borderRadius: '4px',
            minWidth: '20px',
          }}
        >
          {isSidebarCollapsed ? <GoSidebarCollapse /> : <GoSidebarExpand />}
        </Button>
      </div>

      {/* Spacer div for layout */}
      <div
        ref={spacerRef}
        className="flex-shrink-0 z-2 pointer-events-none"
        style={
          windowMinSize.xs
            ? {
                width: 0,
              }
            : {
                width: isSidebarCollapsed ? 0 : `${currentWidth.current}px`,
                transition: 'all 200ms ease-out', // Default transition, overridden during dragging
              }
        }
      />

      {/* Main Content Area */}
      <div
        className="flex-1 flex flex-col z-2 relative overflow-hidden"
        style={{
          backgroundColor: token.colorBgContainer,
        }}
      >
        {/* Toolbar with Traffic Lights */}

        {/* Content */}
        <div className="flex-1">
          <div className="w-full">{children}</div>
        </div>
        {!isSidebarCollapsed && (
          <div
            className="absolute top-0 left-0 w-1 h-full cursor-col-resize"
            onMouseDown={handleMouseDown}
          />
        )}
      </div>
    </div>
  )
}
