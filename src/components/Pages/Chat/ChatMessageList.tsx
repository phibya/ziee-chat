import { memo, useEffect, useRef } from 'react'
import { Flex, Typography } from 'antd'
import { MessageOutlined } from '@ant-design/icons'
import { ChatMessage } from './ChatMessage'
import { useChatStore } from '../../../store'

const { Text } = Typography

export const ChatMessageList = memo(function ChatMessageList() {
  const { messages, sending, isStreaming, streamingMessage, error, loading } =
    useChatStore()
  const messagesEndRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages.length]) // Use length instead of entire array to prevent unnecessary rerenders

  if (!loading && messages.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-center py-20">
        <MessageOutlined className="text-5xl mb-4" />
        <Text className="text-lg">Start your conversation</Text>
      </div>
    )
  }

  return (
    <Flex className={'flex-col gap-1 w-full'}>
      {messages.map(msg => (
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
            files: [],
          }}
        />
      )}
      {error && (
        <ChatMessage
          message={{
            id: 'streaming-error',
            conversation_id: '',
            content: error,
            role: 'assistant',
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString(),
            edit_count: 0,
            originated_from_id: 'streaming-error',
            files: [],
          }}
        />
      )}
      <div ref={messagesEndRef} />
    </Flex>
  )
})
