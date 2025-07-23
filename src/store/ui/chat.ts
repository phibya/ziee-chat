import { create } from 'zustand'

interface ChatUIState {
  // Message editing state
  editingMessageId: string | null
  editingMessageContent: string
  showMessageToolBox: { [messageId: string]: boolean }

  // Chat input state
  inputDisabled: boolean
  inputPlaceholder: string
}

export const useChatUIStore = create<ChatUIState>(() => ({
  // Initial state
  editingMessageId: null,
  editingMessageContent: '',
  showMessageToolBox: {},
  inputDisabled: false,
  inputPlaceholder: '',
}))

// Actions
export const startEditingMessage = (messageId: string, content: string) => {
  useChatUIStore.setState({
    editingMessageId: messageId,
    editingMessageContent: content,
  })
}

export const stopEditingMessage = () => {
  useChatUIStore.setState({
    editingMessageId: null,
    editingMessageContent: '',
  })
}

export const updateEditingContent = (content: string) => {
  useChatUIStore.setState({
    editingMessageContent: content,
  })
}

export const setMessageToolBoxVisible = (
  messageId: string,
  visible: boolean,
) => {
  useChatUIStore.setState(state => ({
    showMessageToolBox: {
      ...state.showMessageToolBox,
      [messageId]: visible,
    },
  }))
}

export const setInputDisabled = (disabled: boolean) => {
  useChatUIStore.setState({ inputDisabled: disabled })
}

export const setInputPlaceholder = (placeholder: string) => {
  useChatUIStore.setState({ inputPlaceholder: placeholder })
}

export const resetChatUI = () => {
  useChatUIStore.setState({
    editingMessageId: null,
    editingMessageContent: '',
    showMessageToolBox: {},
    inputDisabled: false,
    inputPlaceholder: '',
  })
}
