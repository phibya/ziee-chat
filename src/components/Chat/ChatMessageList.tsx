import { memo, useEffect, useRef } from 'react'
import { Flex, Spin, Typography } from 'antd'
import { useTranslation } from 'react-i18next'
import {
  LoadingOutlined,
  MessageOutlined,
  RobotOutlined,
} from '@ant-design/icons'
import { Message } from '../../types/api/chat'
import { ChatMessage } from './ChatMessage'

const { Text } = Typography

interface ChatMessageListProps {
  messages: Message[]
  isLoading: boolean
  isStreaming: boolean
  editingMessage: string | null
  editValue: string
  messageBranches: { [key: string]: Message[] }
  loadingBranches: { [key: string]: boolean }
  onEditMessage: (messageId: string, content: string) => void
  onSaveEdit: () => void
  onCancelEdit: () => void
  onEditValueChange: (value: string) => void
  onLoadBranches: (message: Message) => void
  onSwitchBranch: (messageId: string) => void
}

export const ChatMessageList = memo(function ChatMessageList({
  messages,
  isLoading,
  isStreaming,
  editingMessage,
  editValue,
  messageBranches,
  loadingBranches,
  onEditMessage,
  onSaveEdit,
  onCancelEdit,
  onEditValueChange,
  onLoadBranches,
  onSwitchBranch,
}: ChatMessageListProps) {
  const { t } = useTranslation()
  const messagesEndRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages])

  const getBranchInfo = (msg: Message) => {
    const branchKey = `${msg.id}`
    const branches = messageBranches[branchKey] || []
    const currentIndex = branches.findIndex(b => b.is_active_branch)

    return {
      branches,
      currentIndex,
      hasBranches: branches.length > 1,
      isLoading: loadingBranches[branchKey] || false,
    }
  }

  if (messages.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-center py-20">
        <MessageOutlined className="text-5xl mb-4" />
        <Text className="text-lg">Start your conversation</Text>
      </div>
    )
  }

  return (
    <Flex className={'flex-col gap-3'}>
      {messages.map(message => (
        <ChatMessage
          key={message.id}
          message={message}
          isEditing={editingMessage === message.id}
          editValue={editValue}
          branchInfo={getBranchInfo(message)}
          onEdit={onEditMessage}
          onSaveEdit={onSaveEdit}
          onCancelEdit={onCancelEdit}
          onEditValueChange={onEditValueChange}
          onLoadBranches={onLoadBranches}
          onSwitchBranch={onSwitchBranch}
        />
      ))}

      {(isLoading || isStreaming) && (
        <div className="mb-6">
          <div className="flex items-center gap-3 mb-2">
            <div className="w-8 h-8 rounded-full flex items-center justify-center text-sm font-medium">
              <RobotOutlined />
            </div>
          </div>
          <div className="ml-11">
            <div className="flex items-center gap-2 text-base">
              <Spin
                indicator={<LoadingOutlined style={{ fontSize: 16 }} spin />}
              />
              <span>
                {isStreaming ? t('chat.generating') : t('chat.thinking')}
              </span>
            </div>
          </div>
        </div>
      )}
      <div ref={messagesEndRef} />
    </Flex>
  )
})
