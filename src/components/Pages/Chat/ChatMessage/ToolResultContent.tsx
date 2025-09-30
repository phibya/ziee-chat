import { memo } from 'react'
import { theme, Typography } from 'antd'
import { CheckCircleOutlined, CloseCircleOutlined } from '@ant-design/icons'
import {
  MessageContentItem,
  MessageContentDataToolResult,
} from '../../../../types'

interface ToolResultContentProps {
  content: MessageContentItem
}

export const ToolResultContent = memo(function ToolResultContent({
  content,
}: ToolResultContentProps) {
  const { token } = theme.useToken()
  const toolResultData = content.content as MessageContentDataToolResult
  const isSuccess = toolResultData.success

  return (
    <div
      className="rounded-lg p-3"
      style={{
        border: `1px solid ${isSuccess ? token.colorSuccessBorder : token.colorErrorBorder}`,
        backgroundColor: isSuccess ? token.colorSuccessBg : token.colorErrorBg,
      }}
    >
      <Typography.Text strong>
        {isSuccess ? <CheckCircleOutlined /> : <CloseCircleOutlined />} Tool
        Result
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
})
