import { memo, useEffect, useRef } from 'react'
import { Flex, Typography } from 'antd'
import { MessageOutlined } from '@ant-design/icons'
import { ChatMessage } from './ChatMessage'
import { Stores } from '../../store'

const { Text } = Typography

export const ChatMessageList = memo(function ChatMessageList() {
  const { currentMessages, sending, isStreaming, streamingMessage } =
    Stores.Chat
  const messagesEndRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [currentMessages])

  if (currentMessages.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-center py-20">
        <MessageOutlined className="text-5xl mb-4" />
        <Text className="text-lg">Start your conversation</Text>
      </div>
    )
  }

  return (
    <Flex className={'flex-col gap-1 w-full'}>
      {currentMessages.map(msg => (
        <ChatMessage key={msg.id} message={msg} />
      ))}

      {(sending || isStreaming) && (
        <ChatMessage
          message={{
            id: 'streaming-temp',
            conversation_id: '',
            content: isStreaming && streamingMessage ? streamingMessage : '',
            role: 'assistant',
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString(),
            edit_count: 0,
            originated_from_id: 'streaming-temp',
          }}
        />
      )}
      <div ref={messagesEndRef} />
    </Flex>
  )
})
