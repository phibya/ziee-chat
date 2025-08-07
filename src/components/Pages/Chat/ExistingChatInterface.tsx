import { Flex } from 'antd'
import { useTranslation } from 'react-i18next'
import { useParams } from 'react-router-dom'
import { useChatStore } from '../../../store'
import { ChatHeader } from './ChatHeader'
import { ChatInput } from './ChatInput'
import { ChatMessageList } from './ChatMessageList'

export function ExistingChatInterface() {
  const { conversationId } = useParams<{ conversationId?: string }>()

  if (!conversationId) {
    return null
  }
  const { t } = useTranslation()
  // Chat store
  const { conversation, loading, error } = useChatStore()

  if (loading) {
    return (
      <Flex className="flex-col items-center justify-center h-full">
        <div className="text-lg">{t('chat.loading')}</div>
      </Flex>
    )
  }

  if (error) {
    return (
      <Flex className="flex-col items-center justify-center h-full">
        <div className="text-lg text-red-500">{t('chat.error', { error })}</div>
      </Flex>
    )
  }

  if (!conversation) {
    return <div>Conversation not found</div>
  }

  return (
    <Flex className="flex-col h-dvh gap-1">
      <div className={'top-0 left-0 w-full z-10'}>
        <ChatHeader />
      </div>
      <Flex className={'w-full flex-1 h-full overflow-auto'}>
        <div className={'self-center max-w-4xl w-full h-full m-auto px-4 pt-2'}>
          <ChatMessageList />
        </div>
      </Flex>
      <div className={'w-full pb-2 justify-items-center'}>
        <div className={'max-w-4xl w-full px-2'}>
          <ChatInput />
        </div>
      </div>
    </Flex>
  )
}
