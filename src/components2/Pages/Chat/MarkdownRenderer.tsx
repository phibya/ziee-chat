import { EnhancedMarkdownRenderer } from './EnhancedMarkdownRenderer'

interface MarkdownRendererProps {
  content: string
  className?: string
}

export function MarkdownRenderer({
  content,
  className,
}: MarkdownRendererProps) {
  return <EnhancedMarkdownRenderer content={content} className={className} />
}