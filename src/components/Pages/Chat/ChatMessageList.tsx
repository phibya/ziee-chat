import { memo, useEffect, useRef } from 'react'
import { Flex, Typography } from 'antd'
import { LoadingOutlined, MessageOutlined } from '@ant-design/icons'
import { ChatMessage } from './ChatMessage'
import { useChatStore } from '../../../store'

// Helper function to create structured text content
const createTextContent = (text: string) => [
  {
    id: crypto.randomUUID(),
    message_id: '',
    content_type: 'text' as const,
    content: { text },
    sequence_order: 0,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  },
]

const { Text } = Typography

export const ChatMessageList = memo(function ChatMessageList() {
  const { messages, error, loading, sending, isStreaming } = useChatStore()
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

      {error && (
        <ChatMessage
          message={{
            id: 'streaming-error',
            conversation_id: '',
            contents: createTextContent(error).map(c => ({
              ...c,
              message_id: 'streaming-error',
            })),
            role: 'assistant',
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString(),
            edit_count: 0,
            originated_from_id: 'streaming-error',
            files: [],
          }}
        />
      )}
      {(sending || isStreaming) && (
        <div className={'w-full h-20 mt-3'}>
          <LoadingOutlined spin className={'text-xl'} />
        </div>
      )}

      <div ref={messagesEndRef} />
    </Flex>
  )
})
