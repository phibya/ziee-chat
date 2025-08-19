import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'

interface ChatUIState {
  showTime: boolean
}

export const useChatUIStore = create<ChatUIState>()(
  subscribeWithSelector((_set, _get) => ({
    showTime: false,
  })),
)

// Store methods - defined OUTSIDE the store definition
export const toggleShowTime = () => {
  const currentState = useChatUIStore.getState()
  useChatUIStore.setState({ showTime: !currentState.showTime })
}

export const setShowTime = (show: boolean) => {
  useChatUIStore.setState({ showTime: show })
}
