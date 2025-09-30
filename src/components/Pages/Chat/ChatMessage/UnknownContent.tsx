import { memo } from 'react'
import { theme, Typography } from 'antd'
import { MessageContentItem } from '../../../../types'

interface UnknownContentProps {
  content: MessageContentItem
}

export const UnknownContent = memo(function UnknownContent({
  content,
}: UnknownContentProps) {
  const { token } = theme.useToken()

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
})
