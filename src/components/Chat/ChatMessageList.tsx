import { memo, useEffect, useRef } from 'react'
import { Flex, Spin, Typography } from 'antd'
import { useTranslation } from 'react-i18next'
import { useShallow } from 'zustand/react/shallow'
import {
  LoadingOutlined,
  MessageOutlined,
  RobotOutlined,
} from '@ant-design/icons'
import { ChatMessage } from './ChatMessage'
import { useChatStore } from '../../store/chat'

const { Text } = Typography

export const ChatMessageList = memo(function ChatMessageList() {
  const {
    currentMessages,
    sending,
    isStreaming,
    streamingMessage,
  } = useChatStore(
    useShallow(state => ({
      currentMessages: state.currentMessages,
      sending: state.sending,
      isStreaming: state.isStreaming,
      streamingMessage: state.streamingMessage,
    })),
  )
  const { t } = useTranslation()
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
    <Flex className={'flex-col gap-3 w-full'}>
      {currentMessages.map(msg => (
        <ChatMessage
          key={msg.id}
          message={msg}
        />
      ))}

      {(sending || isStreaming) && (
        <div className="mb-6">
          <div className="flex items-center gap-3 mb-2">
            <div className="w-8 h-8 rounded-full flex items-center justify-center text-sm font-medium">
              <RobotOutlined />
            </div>
          </div>
          <div className="ml-11">
            {isStreaming && streamingMessage ? (
              <div className="text-base">{streamingMessage}</div>
            ) : (
              <div className="flex items-center gap-2 text-base">
                <Spin
                  indicator={<LoadingOutlined style={{ fontSize: 16 }} spin />}
                />
                <span>
                  {isStreaming ? t('chat.generating') : t('chat.thinking')}
                </span>
              </div>
            )}
          </div>
        </div>
      )}
      <div ref={messagesEndRef} />
    </Flex>
  )
})
