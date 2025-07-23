import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import remarkBreaks from 'remark-breaks'
import remarkMath from 'remark-math'
import remarkEmoji from 'remark-emoji'
import rehypeHighlight from 'rehype-highlight'
import rehypeRaw from 'rehype-raw'
import rehypeKatex from 'rehype-katex'

// Import KaTeX CSS for math rendering
import 'katex/dist/katex.min.css'
// Import highlight.js theme
import 'highlight.js/styles/github.css'
// Import custom markdown styles
import { getResolvedAppearanceTheme } from '../../store'

interface EnhancedMarkdownRendererProps {
  content: string
  className?: string
}

// Mermaid component for diagram rendering

// Enhanced code block with copy functionality

export function EnhancedMarkdownRenderer({
  content,
  className,
}: EnhancedMarkdownRendererProps) {
  const resolvedTheme = getResolvedAppearanceTheme()

  return (
    <div className={`classless ${resolvedTheme} ${className || ''}`}>
      <ReactMarkdown
        remarkPlugins={[remarkGfm, remarkBreaks, remarkMath, remarkEmoji]}
        rehypePlugins={[
          [
            rehypeHighlight,
            {
              detect: true,
              ignoreMissing: true,
            },
          ],
          rehypeRaw,
          rehypeKatex,
        ]}
      >
        {content}
      </ReactMarkdown>
    </div>
  )
}
