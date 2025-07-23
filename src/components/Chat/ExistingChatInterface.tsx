import { App, Flex } from 'antd'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useParams } from 'react-router-dom'
import {
  Stores,
  loadUserAssistants,
  loadConversationById,
  clearChatError,
} from '../../store'
import { ChatHeader } from './ChatHeader'
import { ChatInput } from './ChatInput'
import { ChatMessageList } from './ChatMessageList'

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
  } = Stores.Chat

  // Assistants loading state
  const [assistantsLoading, setAssistantsLoading] = useState(false)

  useEffect(() => {
    initializeData()
  }, [])

  useEffect(() => {
    if (conversationId) {
      loadConversationById(conversationId, true)
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
      setAssistantsLoading(true)
      await loadUserAssistants()
    } catch (error: any) {
      message.error(error?.message || t('common.failedToLoadData'))
    } finally {
      setAssistantsLoading(false)
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
