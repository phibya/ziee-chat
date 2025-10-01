import { memo, useState } from 'react'
import { theme, Typography } from 'antd'
import {
  CheckCircleOutlined,
  CloseCircleOutlined,
  DownOutlined,
  RightOutlined,
} from '@ant-design/icons'
import {
  MessageContentItem,
  MessageContentDataToolResult,
} from '../../../../types'
import { DivScrollY } from '../../../common/DivScrollY.tsx'

interface ToolResultContentProps {
  content: MessageContentItem
}

export const ToolResultContent = memo(function ToolResultContent({
  content,
}: ToolResultContentProps) {
  const { token } = theme.useToken()
  const toolResultData = content.content as MessageContentDataToolResult
  const isSuccess = toolResultData.success
  const [isCollapsed, setIsCollapsed] = useState(true)

  return (
    <div
      className="rounded-lg p-3"
      style={{
        border: `1px solid ${isSuccess ? token.colorSuccessBorder : token.colorErrorBorder}`,
        backgroundColor: isSuccess ? token.colorSuccessBg : token.colorErrorBg,
      }}
    >
      <div
        className="flex items-center gap-2 cursor-pointer"
        onClick={() => setIsCollapsed(!isCollapsed)}
      >
        {isCollapsed ? <RightOutlined /> : <DownOutlined />}
        <Typography.Text strong>
          {isSuccess ? <CheckCircleOutlined /> : <CloseCircleOutlined />} Tool
          Result
        </Typography.Text>
      </div>
      {!isCollapsed && (
        <div className="mt-2">
          {toolResultData.error_message && (
            <div className="mt-1">
              <Typography.Text type="danger">
                Error: {toolResultData.error_message}
              </Typography.Text>
            </div>
          )}
          <div className="mt-2">
            <Typography.Text type="secondary">Result:</Typography.Text>
            <DivScrollY
              className="mt-1 p-1 rounded max-h-40"
              style={{
                backgroundColor: token.colorBgContainer,
                border: `1px solid ${token.colorBorderSecondary}`,
                fontSize: '12px',
              }}
            >
              <pre className="w-full rounded">
                {JSON.stringify(toolResultData.result, null, 2)}
              </pre>
            </DivScrollY>
          </div>
        </div>
      )}
    </div>
  )
})
