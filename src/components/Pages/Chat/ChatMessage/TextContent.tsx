import { memo } from 'react'
import { MarkdownRenderer } from '../MarkdownRenderer'
import { MessageContentItem, MessageContentDataText } from '../../../../types'

interface TextContentProps {
  content: MessageContentItem
  isUser: boolean
}

export const TextContent = memo(function TextContent({
  content,
  isUser,
}: TextContentProps) {
  const textData = content.content as MessageContentDataText

  if (!textData.text) {
    return null
  }

  if (isUser) {
    return <div style={{ whiteSpace: 'pre-wrap' }}>{textData.text}</div>
  }

  return (
    <div className={'w-full overflow-hidden pt-2 pl-2'}>
      <MarkdownRenderer content={textData.text.trim()} />
    </div>
  )
})
