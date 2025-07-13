import { useEffect } from 'react'
import { App, Flex } from 'antd'
import { useParams } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { useShallow } from 'zustand/react/shallow'
import { ChatHeader } from './ChatHeader'
import { ChatMessageList } from './ChatMessageList'
import { ChatInput } from './ChatInput'
import { useChatStore } from '../../store/chat'
import { useAssistantsStore } from '../../store/assistants'

export function ExistingChatInterface() {
  const { conversationId } = useParams<{ conversationId?: string }>()

  if (!conversationId) {
    return null
  }
  const { t } = useTranslation()
  const { message } = App.useApp()

  // Chat store
  const {
    currentConversation,
    loading: chatLoading,
    error: chatError,
    loadConversation,
    clearError: clearChatError,
  } = useChatStore(
    useShallow(state => ({
      currentConversation: state.currentConversation,
      loading: state.loading,
      error: state.error,
      loadConversation: state.loadConversation,
      clearError: state.clearError,
    })),
  )

  // Assistants store
  const { loading: assistantsLoading, loadAssistants } = useAssistantsStore(
    useShallow(state => ({
      loading: state.loading,
      loadAssistants: state.loadAssistants,
    })),
  )

  useEffect(() => {
    initializeData()
    return () => {}
  }, [])

  useEffect(() => {
    if (conversationId) {
      loadConversation(conversationId, true)
    }
  }, [conversationId])

  // Show errors
  useEffect(() => {
    if (chatError) {
      message.error(chatError)
      clearChatError()
    }
  }, [chatError, message, clearChatError])

  const initializeData = async () => {
    try {
      await loadAssistants()
    } catch (error: any) {
      message.error(error?.message || t('common.failedToLoadData'))
    }
  }

  if (chatLoading || assistantsLoading) {
    return <div>Loading...</div>
  }

  if (!currentConversation) {
    return <div>Conversation not found</div>
  }

  return (
    <Flex className="flex-col h-dvh gap-3 relative">
      <div className={'absolute top-0 left-0 w-full z-10 backdrop-blur-2xl'}>
        <ChatHeader />
      </div>
      <Flex
        className={
          'max-w-4xl self-center w-full flex-1 h-full overflow-auto !pt-20 !mb-10'
        }
      >
        <ChatMessageList />
      </Flex>
      <div className={'absolute bottom-0 w-full pb-2 justify-items-center'}>
        <div className={'max-w-4xl w-full'}>
          <ChatInput />
        </div>
      </div>
    </Flex>
  )
}
