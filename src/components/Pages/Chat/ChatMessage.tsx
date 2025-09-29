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
import { FileCard } from '../../common/FileCard'
import { Stores, useChatStore } from '../../../store'
import { useMessageBranchStore } from '../../../store/messageBranches.ts'
import {
  Message,
  Permission,
  MessageContentItem,
  MessageContentDataText,
  MessageContentDataToolCall,
  MessageContentDataToolResult,
  MessageContentDataFileAttachment,
  MessageContentDataError,
} from '../../../types'
import dayjs from 'dayjs'
import { PermissionGuard } from '../../Auth/PermissionGuard.tsx'

// Component to render individual content items
const ContentRenderer = memo(function ContentRenderer({
  content,
  isUser,
  token,
}: {
  content: MessageContentItem
  isUser: boolean
  token: any
}) {
  switch (content.content_type) {
    case 'text': {
      const textData = content.content as MessageContentDataText
      if (textData.text) {
        return isUser ? (
          <div style={{ whiteSpace: 'pre-wrap' }}>{textData.text}</div>
        ) : (
          <div className={'w-full overflow-hidden'}>
            <MarkdownRenderer content={textData.text.trim()} />
          </div>
        )
      }
      return null
    }

    case 'tool_call': {
      const toolCallData = content.content as MessageContentDataToolCall
      return (
        <div
          className="rounded-lg p-3"
          style={{
            border: `1px solid ${token.colorBorderSecondary}`,
            backgroundColor: token.colorInfoBg,
          }}
        >
          <Typography.Text strong>
            🔧 Tool Call: {toolCallData.tool_name}
          </Typography.Text>
          <div className="mt-2">
            <Typography.Text type="secondary">
              Call ID: {toolCallData.call_id}
            </Typography.Text>
            <div className="mt-1">
              <Typography.Text type="secondary">Arguments:</Typography.Text>
              <pre
                className="mt-1 p-2 rounded overflow-auto"
                style={{
                  backgroundColor: token.colorBgContainer,
                  border: `1px solid ${token.colorBorderSecondary}`,
                  fontSize: '12px',
                }}
              >
                {JSON.stringify(toolCallData.arguments, null, 2)}
              </pre>
            </div>
          </div>
        </div>
      )
    }

    case 'tool_result': {
      const toolResultData = content.content as MessageContentDataToolResult
      const isSuccess = toolResultData.success
      return (
        <div
          className="rounded-lg p-3"
          style={{
            border: `1px solid ${isSuccess ? token.colorSuccessBorder : token.colorErrorBorder}`,
            backgroundColor: isSuccess
              ? token.colorSuccessBg
              : token.colorErrorBg,
          }}
        >
          <Typography.Text strong>
            {isSuccess ? '✅' : '❌'} Tool Result
          </Typography.Text>
          <div className="mt-2">
            <Typography.Text type="secondary">
              Call ID: {toolResultData.call_id}
            </Typography.Text>
            {toolResultData.error_message && (
              <div className="mt-1">
                <Typography.Text type="danger">
                  Error: {toolResultData.error_message}
                </Typography.Text>
              </div>
            )}
            <div className="mt-2">
              <Typography.Text type="secondary">Result:</Typography.Text>
              <pre
                className="mt-1 p-2 rounded max-h-40 overflow-y-auto"
                style={{
                  backgroundColor: token.colorBgContainer,
                  border: `1px solid ${token.colorBorderSecondary}`,
                  fontSize: '12px',
                }}
              >
                {JSON.stringify(toolResultData.result, null, 2)}
              </pre>
            </div>
          </div>
        </div>
      )
    }

    case 'file_attachment': {
      const fileData = content.content as MessageContentDataFileAttachment
      return (
        <div
          className="rounded-lg p-3"
          style={{
            border: `1px solid ${token.colorBorderSecondary}`,
            backgroundColor: token.colorBgContainer,
          }}
        >
          <Typography.Text strong>📎 File Attachment</Typography.Text>
          <div className="mt-2">
            <Typography.Text type="secondary">
              Filename: {fileData.filename}
            </Typography.Text>
            <br />
            <Typography.Text type="secondary">
              File ID: {fileData.file_id}
            </Typography.Text>
            {fileData.file_type && (
              <>
                <br />
                <Typography.Text type="secondary">
                  Type: {fileData.file_type}
                </Typography.Text>
              </>
            )}
          </div>
        </div>
      )
    }

    case 'error': {
      const errorData = content.content as MessageContentDataError
      return (
        <div
          className="rounded-lg p-3"
          style={{
            border: `1px solid ${token.colorErrorBorder}`,
            backgroundColor: token.colorErrorBg,
          }}
        >
          <Typography.Text strong type="danger">
            ⚠️ Error: {errorData.error_type}
          </Typography.Text>
          <div className="mt-2">
            <Typography.Text type="danger">{errorData.message}</Typography.Text>
          </div>
          {errorData.details && (
            <pre
              className="mt-2 p-2 rounded"
              style={{
                backgroundColor: token.colorBgContainer,
                border: `1px solid ${token.colorErrorBorder}`,
                fontSize: '12px',
              }}
            >
              {JSON.stringify(errorData.details, null, 2)}
            </pre>
          )}
        </div>
      )
    }

    default:
      return (
        <div
          className="rounded-lg p-3"
          style={{
            border: `1px solid ${token.colorBorderSecondary}`,
            backgroundColor: token.colorWarningBg,
          }}
        >
          <Typography.Text type="warning">
            Unknown content type: {content.content_type}
          </Typography.Text>
        </div>
      )
  }
})

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
        className={`flex gap-2 p-2 pr-2 rounded-lg relative w-fit min-w-36 flex-col ${isEditing ? 'w-full' : ''}`}
        style={{
          backgroundColor: isUser ? token.colorBgMask : 'transparent',
          border: isUser ? `1px solid ${token.colorBorderSecondary}` : 'none',
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
              <div className={'w-full flex flex-col gap-2 pr-2'}>
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
                        token={token}
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
