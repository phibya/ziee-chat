import { create } from 'zustand'

interface ViewDownloadModalState {
  open: boolean
  loading: boolean
  downloadId: string | null
  providerType: string | null
}

export const useViewDownloadModalStore = create<ViewDownloadModalState>(() => ({
  open: false,
  loading: false,
  downloadId: null,
  providerType: null,
}))

// Modal actions
export const openViewDownloadModal = (
  downloadId: string,
  providerType: string,
) => {
  useViewDownloadModalStore.setState({
    open: true,
    downloadId,
    providerType,
  })
}

export const closeViewDownloadModal = () => {
  useViewDownloadModalStore.setState({
    open: false,
    loading: false,
    downloadId: null,
    providerType: null,
  })
}

export const setViewDownloadModalLoading = (loading: boolean) => {
  useViewDownloadModalStore.setState({
    loading,
  })
}
