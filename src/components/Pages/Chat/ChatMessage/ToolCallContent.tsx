import { memo } from 'react'
import { theme, Typography } from 'antd'
import { ToolOutlined } from '@ant-design/icons'
import {
  MessageContentItem,
  MessageContentDataToolCall,
} from '../../../../types'

interface ToolCallContentProps {
  content: MessageContentItem
}

export const ToolCallContent = memo(function ToolCallContent({
  content,
}: ToolCallContentProps) {
  const { token } = theme.useToken()
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
        <ToolOutlined /> Tool Call: {toolCallData.tool_name}
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
})
