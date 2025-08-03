import { memo, useState } from 'react'
import { Avatar, Button, Flex, Spin, theme, Typography } from 'antd'
import { useTranslation } from 'react-i18next'
import {
  LeftOutlined,
  LoadingOutlined,
  RightOutlined,
  UserOutlined,
} from '@ant-design/icons'
import { Message, MessageBranch } from '../../types/api/chat'
import { MarkdownRenderer } from './MarkdownRenderer'
import { ChatInput } from './ChatInput'
import { FileCard } from '../common/FileCard'
import {
  Stores,
  loadConversationMessageBranches,
  switchMessageBranch,
  startEditingMessage,
  setMessageToolBoxVisible,
} from '../../store'

interface BranchInfo {
  branches: MessageBranch[]
  currentIndex: number
  hasBranches: boolean
  isLoading: boolean
}

export const ChatMessage = memo(function ChatMessage({
  message,
}: {
  message: Message
}) {
  const { t } = useTranslation()
  const isUser = message.role === 'user'
  const { token } = theme.useToken()

  const { currentConversation } = Stores.Chat

  const { editingMessageId, showMessageToolBox } = Stores.UI.Chat

  const isEditing = editingMessageId === message.id
  const showToolBox = showMessageToolBox[message.id] || false

  const [branchInfo, setBranchInfo] = useState<BranchInfo>({
    branches: [],
    currentIndex: 0,
    hasBranches: false,
    isLoading: false,
  })
  const handleMouseOverOrClick = async (isClicked: boolean = false) => {
    setMessageToolBoxVisible(message.id, !isClicked ? true : !showToolBox)
    if (
      message.edit_count > 0 &&
      !branchInfo.isLoading &&
      !branchInfo.branches.length
    ) {
      setBranchInfo(prev => ({ ...prev, isLoading: true }))

      if (!currentConversation) return

      let branches = await loadConversationMessageBranches(message.id)
      let currentIndex = 0
      for (let i = 0; i < branches.length; i++) {
        if (branches[i].id === currentConversation.active_branch_id) {
          break
        }
        if (!branches[i].is_clone) {
          currentIndex = i + 1
        }
      }

      branches = branches.filter(b => !b.is_clone)

      setBranchInfo({
        branches,
        currentIndex,
        hasBranches: branches.length > 1,
        isLoading: false,
      })
    }
  }

  const handleMouseLeave = () => {
    setMessageToolBoxVisible(message.id, false)
  }

  const handleEdit = () => {
    startEditingMessage({
      messageId: message.id,
      content: message.content,
      files: message.files,
    })
  }

  const handleSwitchBranch = async (branchId: string) => {
    if (!currentConversation) return
    try {
      await switchMessageBranch(currentConversation.id, branchId)
    } catch (error) {
      console.error('Failed to switch branch:', error)
    }
  }

  if (message.content.trim() === '') {
    return null // Skip rendering empty messages
  }

  return (
    <Flex
      key={message.id}
      className={`flex gap-2 !p-2 rounded-lg relative w-fit min-w-36 ${isEditing ? 'w-full' : ''}`}
      style={{
        backgroundColor: isUser ? token.colorBgContainer : 'transparent',
      }}
      onMouseOver={() => handleMouseOverOrClick()}
      onClick={() => handleMouseOverOrClick(true)}
      onMouseLeave={handleMouseLeave}
    >
      <div className={`flex mb-0 ${!isUser ? 'hidden' : ''}`}>
        <Avatar size={24} icon={<UserOutlined />} />
      </div>

      {/* Message content */}
      <Flex className={`${isUser ? '!pt-0.5' : ''} flex-1`}>
        {isEditing ? (
          <ChatInput isEditing={true} />
        ) : (
          <div
            className={'w-full flex flex-col gap-2'}
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

            {/* Render files if message has any */}
            {message.files && message.files.length > 0 && (
              <Flex className="mt-4 gap-2 flex-wrap">
                {message.files.map(file => (
                  <FileCard
                    key={file.id}
                    file={file}
                    size={80}
                    canRemove={false}
                  />
                ))}
              </Flex>
            )}
          </div>
        )}
      </Flex>

      {/* Tools/Actions at the bottom for user messages */}
      {isUser && !isEditing && (
        <div
          className="flex items-center absolute -bottom-2.5 right-2 border rounded-md backdrop-blur-2xl"
          style={{
            borderColor: token.colorBorderSecondary,
            display: showToolBox ? 'flex' : 'none',
          }}
        >
          <Button size="small" type="text" onClick={handleEdit}>
            {t('chat.edit')}
          </Button>

          <Flex
            className={'gap-0'}
            style={{
              display: branchInfo.hasBranches ? 'flex' : 'none',
            }}
          >
            <Button
              size="small"
              type="text"
              icon={<LeftOutlined />}
              disabled={branchInfo.currentIndex <= 0}
              onClick={() => {
                const prevBranch =
                  branchInfo.branches[branchInfo.currentIndex - 1]
                if (prevBranch) {
                  handleSwitchBranch(prevBranch.id)
                  setBranchInfo({
                    ...branchInfo,
                    currentIndex: branchInfo.currentIndex - 1,
                  })
                }
              }}
            />
            <Typography.Text>
              {branchInfo.currentIndex + 1} / {branchInfo.branches.length}
            </Typography.Text>
            <Button
              size="small"
              type="text"
              icon={<RightOutlined />}
              disabled={
                branchInfo.currentIndex >= branchInfo.branches.length - 1
              }
              onClick={() => {
                const nextBranch =
                  branchInfo.branches[branchInfo.currentIndex + 1]
                if (nextBranch) {
                  handleSwitchBranch(nextBranch.id)
                  setBranchInfo({
                    ...branchInfo,
                    currentIndex: branchInfo.currentIndex + 1,
                  })
                }
              }}
            />
          </Flex>
        </div>
      )}
    </Flex>
  )
})
