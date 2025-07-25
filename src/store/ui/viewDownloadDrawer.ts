import { create } from 'zustand'

interface ViewDownloadModalState {
  open: boolean
  loading: boolean
  downloadId: string | null
}

export const useViewDownloadModalStore = create<ViewDownloadModalState>(() => ({
  open: false,
  loading: false,
  downloadId: null,
}))

// Modal actions
export const openViewDownloadModal = (downloadId: string) => {
  useViewDownloadModalStore.setState({
    open: true,
    downloadId,
  })
}

export const closeViewDownloadModal = () => {
  useViewDownloadModalStore.setState({
    open: false,
    loading: false,
    downloadId: null,
  })
}

export const setViewDownloadModalLoading = (loading: boolean) => {
  useViewDownloadModalStore.setState({
    loading,
  })
}
