import { create } from 'zustand'

interface LayoutUIState {
  // Mobile/responsive state
  isMobile: boolean
  mobileOverlayOpen: boolean
}

export const useLayoutUIStore = create<LayoutUIState>(() => ({
  // Initial state
  isMobile: false,
  mobileOverlayOpen: false,
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
