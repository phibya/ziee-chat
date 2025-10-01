import { memo, useState } from 'react'
import { theme, Typography } from 'antd'
import { DownOutlined, RightOutlined, ToolOutlined } from '@ant-design/icons'
import {
  MessageContentDataToolCall,
  MessageContentItem,
} from '../../../../types'
import { DivScrollY } from '../../../common/DivScrollY.tsx'

interface ToolCallContentProps {
  content: MessageContentItem
}

export const ToolCallContent = memo(function ToolCallContent({
  content,
}: ToolCallContentProps) {
  const { token } = theme.useToken()
  const toolCallData = content.content as MessageContentDataToolCall
  const [isCollapsed, setIsCollapsed] = useState(true)

  return (
    <div
      className="rounded-lg p-3"
      style={{
        border: `1px solid ${token.colorBorderSecondary}`,
        backgroundColor: token.colorInfoBg,
      }}
    >
      <div
        className="flex items-center gap-2 cursor-pointer"
        onClick={() => setIsCollapsed(!isCollapsed)}
      >
        {isCollapsed ? <RightOutlined /> : <DownOutlined />}
        <Typography.Text strong>
          <ToolOutlined /> Tool Call: {toolCallData.tool_name}
        </Typography.Text>
      </div>
      {!isCollapsed && (
        <div className="mt-2">
          <Typography.Text type="secondary">Arguments:</Typography.Text>
          <DivScrollY
            className="mt-1 p-1 rounded max-h-40"
            style={{
              backgroundColor: token.colorBgContainer,
              border: `1px solid ${token.colorBorderSecondary}`,
              fontSize: '12px',
            }}
          >
            <pre className="w-full rounded">
              {JSON.stringify(toolCallData.arguments, null, 2)}
            </pre>
          </DivScrollY>
        </div>
      )}
    </div>
  )
})
