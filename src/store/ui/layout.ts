import { create } from 'zustand'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
import { isTauriView } from '../../api/core'

interface LayoutUIState {
  // Mobile/responsive state
  isMobile: boolean
  isFullscreen: boolean
  mobileOverlayOpen: boolean
  // Sidebar state
  isSidebarCollapsed: boolean
  mainContentWidth: number
}

export const useLayoutUIStore = create<LayoutUIState>(() => ({
  // Initial state
  isMobile: false,
  isFullscreen: false,
  mobileOverlayOpen: false,
  isSidebarCollapsed: false,
  mainContentWidth: 1000,
}))

// Actions
export const setIsMobile = (isMobile: boolean) => {
  useLayoutUIStore.setState({ isMobile })
}

export const setMobileOverlayOpen = (open: boolean) => {
  useLayoutUIStore.setState({ mobileOverlayOpen: open })
}

export const closeMobileOverlay = () => {
  useLayoutUIStore.setState({ mobileOverlayOpen: false })
}

export const setSidebarCollapsed = (collapsed: boolean) => {
  useLayoutUIStore.setState({ isSidebarCollapsed: collapsed })
}

export const toggleSidebar = () => {
  const currentState = useLayoutUIStore.getState()
  useLayoutUIStore.setState({
    isSidebarCollapsed: !currentState.isSidebarCollapsed,
  })
}

export const setMainContentWidth = (width: number) => {
  useLayoutUIStore.setState({ mainContentWidth: width })
}

export const setIsFullscreen = (isFullscreen: boolean) => {
  useLayoutUIStore.setState({ isFullscreen })
}

// Initialize fullscreen listener for desktop app
let isListenerInitialized = false

export const initializeFullscreenListener = async () => {
  if (!isTauriView || isListenerInitialized) return

  isListenerInitialized = true

  const checkFullscreenState = async () => {
    try {
      const webview = getCurrentWebviewWindow()
      const isFullscreen = await webview.isFullscreen()
      setIsFullscreen(isFullscreen)
    } catch (error) {
      console.error('Failed to check fullscreen state:', error)
    }
  }

  // Check initial state
  await checkFullscreenState()

  // Listen to resize events
  const webview = getCurrentWebviewWindow()
  await webview.listen('tauri://resize', async () => {
    await checkFullscreenState()
  })
}

initializeFullscreenListener()
