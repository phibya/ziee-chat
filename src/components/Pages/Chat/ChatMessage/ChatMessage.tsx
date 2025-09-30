import { memo, useState } from 'react'
import { Avatar, Button, Flex, Spin, theme, Typography } from 'antd'
import { useTranslation } from 'react-i18next'
import {
  LeftOutlined,
  LoadingOutlined,
  RightOutlined,
  UserOutlined,
} from '@ant-design/icons'
import { ChatInput } from '../ChatInput'
import { FileCard } from '../../../common/FileCard'
import { Stores, useChatStore } from '../../../../store'
import { useMessageBranchStore } from '../../../../store/messageBranches.ts'
import { Message, Permission } from '../../../../types'
import dayjs from 'dayjs'
import { PermissionGuard } from '../../../Auth/PermissionGuard.tsx'
import { ContentRenderer } from './ContentRenderer'

export const ChatMessage = memo(function ChatMessage({
  message,
}: {
  message: Message
}) {
  const { t } = useTranslation()
  const isUser = message.role === 'user'
  const { token } = theme.useToken()
  const { showTime } = Stores.UI.ChatUI

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

  // Check if message has any content to render
  if (!message.contents || message.contents.length === 0) {
    return null // Skip rendering empty messages
  }

  return (
    <div className={'w-full flex flex-col overflow-visible'}>
      {/* Render files if message has any */}
      {!isEditing && isUser && message.files && message.files.length > 0 && (
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
        className={`flex gap-2 rounded-lg relative min-w-36 flex-col ${isEditing ? 'w-full' : ''}`}
        style={{
          backgroundColor: isUser ? token.colorBgMask : 'transparent',
          border: isUser ? `1px solid ${token.colorBorderSecondary}` : 'none',
          width: isUser ? 'fit-content' : '100%',
          padding: isUser ? '8px 8px' : '0px',
        }}
        onMouseOver={() => handleMouseOverOrClick()}
        onClick={() => handleMouseOverOrClick(true)}
        onMouseLeave={handleMouseLeave}
      >
        <div className={'flex items-start gap-2 w-full relative'}>
          <div className={`flex ${!isUser ? 'hidden' : ''}`}>
            <Avatar size={24} icon={<UserOutlined />} />
          </div>

          {/* Message content */}
          <div
            className={`${isUser ? '!pt-0.5' : ''} flex flex-1 -mt-[2px] w-full overflow-x-hidden flex-col`}
          >
            {isEditing ? (
              <div className={'w-full flex flex-col gap-2'}>
                <ChatInput
                  editingMessage={message}
                  onDoneEditing={() => setIsEditing(false)}
                />
                <Typography.Text type={'secondary'}>
                  Editing message will create a new branch. You can switch
                  branches using the arrows.
                </Typography.Text>
              </div>
            ) : (
              <div className={'w-full flex flex-col gap-2'}>
                {message.id === 'streaming-temp' &&
                message.contents.length === 0 ? (
                  <Spin indicator={<LoadingOutlined spin />} />
                ) : (
                  message.contents
                    .sort(
                      (a, b) =>
                        new Date(a.created_at).getTime() -
                        new Date(b.created_at).getTime(),
                    )
                    .map((content, index) => (
                      <ContentRenderer
                        key={`${content.id || index}`}
                        content={content}
                        isUser={isUser}
                      />
                    ))
                )}
              </div>
            )}
          </div>
        </div>
        {showTime && !isEditing && (
          <div
            className={'w-full flex justify-end'}
            style={{
              marginTop: isUser ? '-8px' : '-16px',
            }}
          >
            <Typography.Text type={'secondary'} className={'!text-xs'}>
              {dayjs(message.created_at).format('MMM DD, HH:mm:ss')}
            </Typography.Text>
          </div>
        )}
        {/* Tools/Actions at the bottom for user messages */}
        {isUser && !isEditing && (
          <div
            className="flex items-center absolute -bottom-2.5 right-2 border rounded-md backdrop-blur-2xl"
            style={{
              borderColor: token.colorBorderSecondary,
              display: isToolBoxVisible ? 'flex' : 'none',
            }}
          >
            <PermissionGuard
              permissions={[Permission.ChatCreate, Permission.ChatEdit]}
            >
              <Button size="small" type="text" onClick={handleEdit}>
                {t('chat.edit')}
              </Button>
            </PermissionGuard>

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
