import { memo } from 'react'
import { theme, Typography } from 'antd'
import { WarningOutlined } from '@ant-design/icons'
import { MessageContentItem, MessageContentDataError } from '../../../../types'

interface ErrorContentProps {
  content: MessageContentItem
}

export const ErrorContent = memo(function ErrorContent({
  content,
}: ErrorContentProps) {
  const { token } = theme.useToken()
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
        <WarningOutlined /> Error: {errorData.error_type}
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
})
