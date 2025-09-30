import { memo } from 'react'
import { theme, Typography } from 'antd'
import { StopOutlined } from '@ant-design/icons'
import {
  MessageContentItem,
  MessageContentDataToolCallPendingApprovalCancel,
} from '../../../../types'

interface ToolCallPendingApprovalCancelContentProps {
  content: MessageContentItem
}

export const ToolCallPendingApprovalCancelContent = memo(
  function ToolCallPendingApprovalCancelContent({
    content,
  }: ToolCallPendingApprovalCancelContentProps) {
    const { token } = theme.useToken()
    const cancelData =
      content.content as MessageContentDataToolCallPendingApprovalCancel

    return (
      <div
        className="rounded-lg p-3"
        style={{
          border: `1px solid ${token.colorErrorBorder}`,
          backgroundColor: token.colorErrorBg,
        }}
      >
        <Typography.Text strong type="danger">
          <StopOutlined /> Tool Call Cancelled: {cancelData.tool_name}
        </Typography.Text>
        <div className="mt-2">
          <Typography.Text type="secondary">
            The tool execution was cancelled by the user.
          </Typography.Text>
        </div>
      </div>
    )
  },
)
