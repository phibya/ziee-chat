import { create } from 'zustand'

interface LayoutUIState {
  // Mobile/responsive state
  isMobile: boolean
  mobileOverlayOpen: boolean
  // Sidebar state
  isSidebarCollapsed: boolean
}

export const useLayoutUIStore = create<LayoutUIState>(() => ({
  // Initial state
  isMobile: false,
  mobileOverlayOpen: false,
  isSidebarCollapsed: false,
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
