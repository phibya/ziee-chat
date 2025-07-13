import { memo, useState } from 'react'
import { Avatar, Button, Flex, theme, Typography } from 'antd'
import { useTranslation } from 'react-i18next'
import { LeftOutlined, RightOutlined, UserOutlined } from '@ant-design/icons'
import { Message, MessageBranch } from '../../types/api/chat'
import { MarkdownRenderer } from './MarkdownRenderer'
import { MessageEditor } from './MessageEditor'
import { useShallow } from 'zustand/react/shallow'
import { useChatStore } from '../../store/chat.ts'

interface ChatMessageProps {
  message: Message
}

interface BranchInfo {
  branches: MessageBranch[]
  currentIndex: number
  hasBranches: boolean
  isLoading: boolean
}

export const ChatMessage = memo(function ChatMessage({
  message,
}: ChatMessageProps) {
  const { t } = useTranslation()
  const isUser = message.role === 'user'
  const { token } = theme.useToken()
  const [showToolBox, setShowToolBox] = useState(false)

  const { loadMessageBranches, editMessage, switchBranch, currentConversation } = useChatStore(
    useShallow(state => ({
      loadMessageBranches: state.loadMessageBranches,
      editMessage: state.editMessage,
      switchBranch: state.switchBranch,
      currentConversation: state.currentConversation,
    })),
  )

  // Local editing state
  const [isEditing, setIsEditing] = useState(false)
  const [editValue, setEditValue] = useState(message.content)

  const [branchInfo, setBranchInfo] = useState<BranchInfo>({
    branches: [],
    currentIndex: 0,
    hasBranches: false,
    isLoading: false,
  })
  const handleMouseOverOrClick = async (isClicked: boolean = false) => {
    setShowToolBox(!isClicked ? true : !showToolBox)
    if (
      message.edit_count > 0 &&
      !branchInfo.isLoading &&
      !branchInfo.branches.length
    ) {
      setBranchInfo(prev => ({ ...prev, isLoading: true }))

      if (!currentConversation) return

      let branches = await loadMessageBranches(message.id)
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
    setShowToolBox(false)
  }

  const handleEdit = () => {
    setIsEditing(true)
    setEditValue(message.content)
  }

  const handleSaveEdit = async () => {
    try {
      await editMessage(message.id, editValue)
      setIsEditing(false)
    } catch (error) {
      console.error('Failed to save edit:', error)
    }
  }

  const handleCancelEdit = () => {
    setIsEditing(false)
    setEditValue(message.content)
  }

  const handleSwitchBranch = async (branchId: string) => {
    if (!currentConversation) return
    try {
      await switchBranch(currentConversation.id, branchId)
    } catch (error) {
      console.error('Failed to switch branch:', error)
    }
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
      <div className={`flex items-center mb-2 ${!isUser ? 'hidden' : ''}`}>
        <div className="flex items-center">
          <Avatar size={24} icon={<UserOutlined />} />
        </div>
      </div>

      {/* Message content */}
      <Flex className={`${isUser ? '!pt-0.5' : ''} w-full`}>
        {isEditing ? (
          <MessageEditor
            value={editValue}
            onChange={setEditValue}
            onSave={handleSaveEdit}
            onCancel={handleCancelEdit}
          />
        ) : (
          <div
            className={'w-full'}
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
        <div
          className="flex items-center absolute -bottom-2.5 right-2 border rounded-md backdrop-blur-2xl"
          style={{
            borderColor: token.colorBorderSecondary,
            display: showToolBox ? 'flex' : 'none',
          }}
        >
          <Button
            size="small"
            type="text"
            onClick={handleEdit}
          >
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
