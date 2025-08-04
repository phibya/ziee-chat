import { Flex } from 'antd'
import { useTranslation } from 'react-i18next'
import { useParams } from 'react-router-dom'
import { useChatStore } from '../../store'
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
