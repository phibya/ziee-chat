import { memo, useState } from 'react'
import { Avatar, Button, Flex, Spin, theme, Typography } from 'antd'
import { useTranslation } from 'react-i18next'
import {
  LeftOutlined,
  LoadingOutlined,
  RightOutlined,
  UserOutlined,
} from '@ant-design/icons'
import { MarkdownRenderer } from './MarkdownRenderer'
import { ChatInput } from './ChatInput'
import { FileCard } from '../../Common/FileCard'
import { useChatStore } from '../../../store'
import { useMessageBranchStore } from '../../../store/messageBranches.ts'
import { Message } from '../../../types'

export const ChatMessage = memo(function ChatMessage({
  message,
}: {
  message: Message
}) {
  const { t } = useTranslation()
  const isUser = message.role === 'user'
  const { token } = theme.useToken()

  const { activeBranchId, switchBranch, isStreaming } = useChatStore()
  const { branches, loadBranches } = useMessageBranchStore(
    message.id,
    message.originated_from_id,
  )
  const [isEditing, setIsEditing] = useState(false)
  const [isToolBoxVisible, setMessageToolBoxVisible] = useState(false)

  const branchIndex = branches.findIndex(b => b.id === activeBranchId)

  const handleMouseOverOrClick = async (isClicked: boolean = false) => {
    if (isStreaming) return
    setMessageToolBoxVisible(p => (!isClicked ? true : !p))
    if (message.edit_count > 0 && branches.length === 0) {
      loadBranches()
    }
  }

  const handleMouseLeave = () => {
    setMessageToolBoxVisible(false)
  }

  const handleEdit = () => {
    setIsEditing(true)
  }

  const handleSwitchBranch = async (branchId: string) => {
    try {
      await switchBranch(branchId)
    } catch (error) {
      console.error('Failed to switch branch:', error)
    }
  }

  if (message.content.trim() === '') {
    return null // Skip rendering empty messages
  }

  return (
    <div className={'w-full flex flex-col overflow-visible'}>
      {/* Render files if message has any */}
      {isUser && message.files && message.files.length > 0 && (
        <div className="flex gap-2 w-full overflow-x-auto pb-2 !pl-2">
          {message.files.map(file => (
            <div className={'flex-1 min-w-18 max-w-24'}>
              <FileCard
                key={file.id}
                file={file}
                canRemove={false}
                showFileName={false}
              />
            </div>
          ))}
        </div>
      )}
      <div
        key={message.id}
        className={`flex gap-2 p-2 pr-2 rounded-lg relative w-fit min-w-36 ${isEditing ? 'w-full' : ''}`}
        style={{
          backgroundColor: isUser ? token.colorBgMask : 'transparent',
          border: isUser ? `1px solid ${token.colorBorderSecondary}` : 'none',
        }}
        onMouseOver={() => handleMouseOverOrClick()}
        onClick={() => handleMouseOverOrClick(true)}
        onMouseLeave={handleMouseLeave}
      >
        <div className={`flex ${!isUser ? 'hidden' : ''}`}>
          <Avatar size={24} icon={<UserOutlined />} />
        </div>

        {/* Message content */}
        <div
          className={`${isUser ? '!pt-0.5' : ''} flex flex-1 -mt-[2px] w-full overflow-x-hidden`}
        >
          {isEditing ? (
            <ChatInput
              editingMessage={message}
              onDoneEditing={() => setIsEditing(false)}
            />
          ) : (
            <div
              className={'w-full flex flex-col gap-2 pr-2'}
              style={{
                whiteSpace: isUser ? 'pre-wrap' : 'normal',
              }}
            >
              {isUser ? (
                message.content
              ) : message.id === 'streaming-temp' && !message.content ? (
                <Spin indicator={<LoadingOutlined spin />} />
              ) : (
                <MarkdownRenderer content={message.content.trim()} />
              )}
            </div>
          )}
        </div>

        {/* Tools/Actions at the bottom for user messages */}
        {isUser && !isEditing && (
          <div
            className="flex items-center absolute -bottom-2.5 right-2 border rounded-md backdrop-blur-2xl"
            style={{
              borderColor: token.colorBorderSecondary,
              display: isToolBoxVisible ? 'flex' : 'none',
            }}
          >
            <Button size="small" type="text" onClick={handleEdit}>
              {t('chat.edit')}
            </Button>

            <Flex
              className={'gap-0'}
              style={{
                display: branches.length > 1 ? 'flex' : 'none',
              }}
            >
              <Button
                size="small"
                type="text"
                icon={<LeftOutlined />}
                disabled={branchIndex <= 0}
                onClick={() => {
                  const prevBranch = branches[branchIndex - 1]
                  if (prevBranch) {
                    handleSwitchBranch(prevBranch.id)
                  }
                }}
              />
              <Typography.Text>
                {branchIndex + 1} / {branches.length}
              </Typography.Text>
              <Button
                size="small"
                type="text"
                icon={<RightOutlined />}
                disabled={branchIndex >= branches.length - 1}
                onClick={() => {
                  const nextBranch = branches[branchIndex + 1]
                  if (nextBranch) {
                    handleSwitchBranch(nextBranch.id)
                  }
                }}
              />
            </Flex>
          </div>
        )}
      </div>
    </div>
  )
})
