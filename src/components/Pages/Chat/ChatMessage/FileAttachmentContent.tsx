import { memo } from 'react'
import { theme, Typography } from 'antd'
import { PaperClipOutlined } from '@ant-design/icons'
import {
  MessageContentItem,
  MessageContentDataFileAttachment,
} from '../../../../types'

interface FileAttachmentContentProps {
  content: MessageContentItem
}

export const FileAttachmentContent = memo(function FileAttachmentContent({
  content,
}: FileAttachmentContentProps) {
  const { token } = theme.useToken()
  const fileData = content.content as MessageContentDataFileAttachment

  return (
    <div
      className="rounded-lg p-3"
      style={{
        border: `1px solid ${token.colorBorderSecondary}`,
        backgroundColor: token.colorBgContainer,
      }}
    >
      <Typography.Text strong>
        <PaperClipOutlined /> File Attachment
      </Typography.Text>
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
})
