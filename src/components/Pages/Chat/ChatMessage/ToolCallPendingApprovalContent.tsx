import { memo, useState } from 'react'
import { theme, Typography, Button, Flex, App } from 'antd'
import {
  ClockCircleOutlined,
  CheckOutlined,
  CloseOutlined,
} from '@ant-design/icons'
import {
  MessageContentItem,
  MessageContentDataToolCallPendingApproval,
} from '../../../../types'
import { useChatStore } from '../../../../store'
import { createConversationApproval } from '../../../../store/mcpApprovals'

interface ToolCallPendingApprovalContentProps {
  content: MessageContentItem
}

export const ToolCallPendingApprovalContent = memo(
  function ToolCallPendingApprovalContent({
    content,
  }: ToolCallPendingApprovalContentProps) {
    const { token } = theme.useToken()
    const { message } = App.useApp()
    const chatStore = useChatStore()
    const { conversation, sendMessage } = chatStore
    const [loading, setLoading] = useState<
      'once' | 'conversation' | 'deny' | null
    >(null)
    const [isHidden, setIsHidden] = useState(false)

    const pendingData =
      content.content as MessageContentDataToolCallPendingApproval

    // Hide if already approved or denied (is_approved is true or false, not null/undefined)
    if (typeof pendingData.is_approved === 'boolean') {
      return null
    }

    const handleApprove = async (type: 'once' | 'conversation') => {
      if (!conversation) return

      setLoading(type)
      try {
        const expiresAt =
          type === 'once'
            ? new Date(Date.now() + 5000) // 5 seconds
            : new Date(Date.now() + 7 * 24 * 60 * 60 * 1000) // 7 days

        await createConversationApproval(conversation.id, {
          server_id: pendingData.server_id,
          tool_name: pendingData.tool_name,
          approved: true,
          expires_at: expiresAt.toISOString(),
          notes:
            type === 'once'
              ? 'One-time approval'
              : 'Approved for this conversation (7 days)',
          approval_message_content_id: content.id,
        })

        message.success(
          type === 'once'
            ? 'Tool approved for this request'
            : 'Tool approved for this conversation',
        )

        // Hide the component
        setIsHidden(true)

        // Resume the chat - backend will automatically detect pending approval and resume
        if (conversation.model_id && conversation.assistant_id) {
          await sendMessage({
            content: '',
            model_id: conversation.model_id,
            assistant_id: conversation.assistant_id,
            file_ids: [],
            message_id: content.message_id,
          })
        }
      } catch (error) {
        console.error('Failed to approve tool:', error)
        message.error('Failed to approve tool')
      } finally {
        setLoading(null)
      }
    }

    const handleDeny = async () => {
      if (!conversation) return

      setLoading('deny')
      try {
        await createConversationApproval(conversation.id, {
          server_id: pendingData.server_id,
          tool_name: pendingData.tool_name,
          approved: false,
          expires_at: undefined,
          notes: 'User denied tool execution',
          approval_message_content_id: content.id,
        })

        message.info('Tool execution denied')

        // Hide the component
        setIsHidden(true)
      } catch (error) {
        console.error('Failed to deny tool:', error)
        message.error('Failed to deny tool')
      } finally {
        setLoading(null)
      }
    }

    // Don't render if hidden
    if (isHidden) {
      return null
    }

    return (
      <div
        className="rounded-lg p-3 w-full"
        style={{
          border: `1px solid ${token.colorWarningBorder}`,
          backgroundColor: token.colorWarningBg,
        }}
      >
        <Typography.Text strong>
          <ClockCircleOutlined /> Tool Call Pending Approval:{' '}
          {pendingData.tool_name}
        </Typography.Text>
        <div className="flex gap-2 mt-2 flex-col">
          <Typography.Text type="secondary">
            This tool requires your approval before execution.
          </Typography.Text>
          <div>
            <Typography.Text type="secondary">Arguments:</Typography.Text>
            <pre
              className="mt-1 p-2 rounded overflow-auto max-h-40"
              style={{
                backgroundColor: token.colorBgContainer,
                border: `1px solid ${token.colorBorderSecondary}`,
                fontSize: '12px',
              }}
            >
              {JSON.stringify(pendingData.arguments, null, 2)}
            </pre>
          </div>

          <Flex gap={8} wrap="wrap" className="w-full">
            <Button
              type="primary"
              icon={<CheckOutlined />}
              onClick={() => handleApprove('once')}
              loading={loading === 'once'}
              disabled={loading !== null && loading !== 'once'}
            >
              Approve Once
            </Button>
            <Button
              type="default"
              icon={<CheckOutlined />}
              onClick={() => handleApprove('conversation')}
              loading={loading === 'conversation'}
              disabled={loading !== null && loading !== 'conversation'}
            >
              Approve For This Conversation
            </Button>
            <Button
              danger
              icon={<CloseOutlined />}
              onClick={handleDeny}
              loading={loading === 'deny'}
              disabled={loading !== null && loading !== 'deny'}
            >
              Deny
            </Button>
          </Flex>
        </div>
      </div>
    )
  },
)
