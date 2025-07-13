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
  isEditing: boolean
  editValue: string
  onEdit: (messageId: string, content: string) => void
  onSaveEdit: () => void
  onCancelEdit: () => void
  onEditValueChange: (value: string) => void
  onLoadBranches: (message: Message) => void
  onSwitchBranch: (messageId: string) => void
}

interface BranchInfo {
  branches: MessageBranch[]
  currentIndex: number
  hasBranches: boolean
  isLoading: boolean
}

export const ChatMessage = memo(function ChatMessage({
  message,
  isEditing,
  editValue,
  onEdit,
  onSaveEdit,
  onCancelEdit,
  onEditValueChange,
  onSwitchBranch,
}: ChatMessageProps) {
  const { t } = useTranslation()
  const isUser = message.role === 'user'
  const { token } = theme.useToken()
  const [showToolBox, setShowToolBox] = useState(false)

  const { loadMessageBranches } = useChatStore(
    useShallow(state => ({
      loadMessageBranches: state.loadMessageBranches,
    })),
  )

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

      const currentConversation = useChatStore.getState().currentConversation!

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
            onChange={onEditValueChange}
            onSave={onSaveEdit}
            onCancel={onCancelEdit}
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
            onClick={() => onEdit(message.id, message.content)}
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
                  onSwitchBranch(prevBranch.id)
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
                  onSwitchBranch(nextBranch.id)
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
