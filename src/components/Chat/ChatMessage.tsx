import { memo } from 'react'
import { Avatar, Button, Flex, theme } from 'antd'
import { useTranslation } from 'react-i18next'
import { UserOutlined } from '@ant-design/icons'
import { Message } from '../../types/api/chat'
import { MarkdownRenderer } from './MarkdownRenderer'
import { MessageBranches } from './MessageBranches'
import { MessageEditor } from './MessageEditor'

interface ChatMessageProps {
  message: Message
  isEditing: boolean
  editValue: string
  branchInfo: {
    branches: Message[]
    currentIndex: number
    hasBranches: boolean
    isLoading: boolean
  }
  onEdit: (messageId: string, content: string) => void
  onSaveEdit: () => void
  onCancelEdit: () => void
  onEditValueChange: (value: string) => void
  onLoadBranches: (message: Message) => void
  onSwitchBranch: (messageId: string) => void
}

export const ChatMessage = memo(function ChatMessage({
  message,
  isEditing,
  editValue,
  branchInfo,
  onEdit,
  onSaveEdit,
  onCancelEdit,
  onEditValueChange,
  onLoadBranches,
  onSwitchBranch,
}: ChatMessageProps) {
  const { t } = useTranslation()
  const isUser = message.role === 'user'
  const { token } = theme.useToken()

  return (
    <Flex
      key={message.id}
      className={`flex gap-2 !p-2 rounded-lg`}
      style={{
        backgroundColor: isUser ? token.colorBgElevated : 'transparent',
      }}
    >
      <div className={`flex items-center mb-2 ${!isUser ? 'hidden' : ''}`}>
        <div className="flex items-center">
          <Avatar size={24} icon={<UserOutlined />} />
        </div>
      </div>

      {/* Message content */}
      <Flex className={`${isUser ? '!pt-0.5' : ''}`}>
        {isEditing ? (
          <MessageEditor
            value={editValue}
            onChange={onEditValueChange}
            onSave={onSaveEdit}
            onCancel={onCancelEdit}
          />
        ) : (
          <div
            style={{
              whiteSpace: isUser ? 'pre-wrap' : 'normal',
            }}
          >
            {isUser ? (
              message.content
            ) : (
              <MarkdownRenderer content={message.content} />
            )}
          </div>
        )}
      </Flex>

      {/* Tools/Actions at the bottom for user messages */}
      {isUser && !isEditing && (
        <div className="ml-11 mt-2">
          <div className="flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
            <Button
              size="small"
              type="text"
              onClick={() => onEdit(message.id, message.content)}
            >
              {t('chat.edit')}
            </Button>

            <MessageBranches
              branchInfo={branchInfo}
              onLoadBranches={() => onLoadBranches(message)}
              onSwitchBranch={onSwitchBranch}
            />
          </div>
        </div>
      )}
    </Flex>
  )
})
