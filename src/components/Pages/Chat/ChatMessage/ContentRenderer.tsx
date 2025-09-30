import { memo } from 'react'
import { MessageContentItem } from '../../../../types'
import { TextContent } from './TextContent'
import { ToolCallContent } from './ToolCallContent'
import { ToolCallPendingApprovalContent } from './ToolCallPendingApprovalContent'
import { ToolCallPendingApprovalCancelContent } from './ToolCallPendingApprovalCancelContent'
import { ToolResultContent } from './ToolResultContent'
import { FileAttachmentContent } from './FileAttachmentContent'
import { ErrorContent } from './ErrorContent'
import { UnknownContent } from './UnknownContent'

interface ContentRendererProps {
  content: MessageContentItem
  isUser: boolean
}

export const ContentRenderer = memo(function ContentRenderer({
  content,
  isUser,
}: ContentRendererProps) {
  switch (content.content_type) {
    case 'text':
      return <TextContent content={content} isUser={isUser} />

    case 'tool_call':
      return <ToolCallContent content={content} />

    case 'tool_call_pending_approval':
      return <ToolCallPendingApprovalContent content={content} />

    case 'tool_call_pending_approval_cancel':
      return <ToolCallPendingApprovalCancelContent content={content} />

    case 'tool_result':
      return <ToolResultContent content={content} />

    case 'file_attachment':
      return <FileAttachmentContent content={content} />

    case 'error':
      return <ErrorContent content={content} />

    default:
      return <UnknownContent content={content} />
  }
})
