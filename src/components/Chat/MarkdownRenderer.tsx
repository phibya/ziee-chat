import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import remarkBreaks from 'remark-breaks'
import rehypeHighlight from 'rehype-highlight'
import rehypeRaw from 'rehype-raw'
import { Divider, Typography } from 'antd'
import 'prismjs/themes/prism-tomorrow.css'

const { Text, Title, Link } = Typography

interface MarkdownRendererProps {
  content: string
  className?: string
}

export function MarkdownRenderer({
  content,
  className,
}: MarkdownRendererProps) {
  return (
    <div className={className}>
      <ReactMarkdown
        remarkPlugins={[remarkGfm, remarkBreaks]}
        rehypePlugins={[rehypeHighlight, rehypeRaw]}
        components={{
          // Custom components using Ant Design theme
          p: ({ children }) => <Text>{children}</Text>,
          code: ({ inline, className, children, ...props }: any) => {
            if (inline) {
              return <code {...props}>{children}</code>
            }
            return (
              <pre className="overflow-auto">
                <code className={className} {...props}>
                  {children}
                </code>
              </pre>
            )
          },
          blockquote: ({ children }) => (
            <blockquote>
              <Text type="secondary" italic>
                {children}
              </Text>
            </blockquote>
          ),
          h1: ({ children }) => <Title level={2}>{children}</Title>,
          h2: ({ children }) => <Title level={3}>{children}</Title>,
          h3: ({ children }) => <Title level={4}>{children}</Title>,
          ul: ({ children }) => <ul>{children}</ul>,
          ol: ({ children }) => <ol>{children}</ol>,
          li: ({ children }) => (
            <li>
              <Text>{children}</Text>
            </li>
          ),
          strong: ({ children }) => <Text strong>{children}</Text>,
          em: ({ children }) => <Text italic>{children}</Text>,
          a: ({ href, children }) => (
            <Link href={href} target="_blank" rel="noopener noreferrer">
              {children}
            </Link>
          ),
          table: ({ children }) => (
            <div className="overflow-x-auto">
              <table>{children}</table>
            </div>
          ),
          th: ({ children }) => (
            <th>
              <Text strong>{children}</Text>
            </th>
          ),
          td: ({ children }) => (
            <td>
              <Text>{children}</Text>
            </td>
          ),
          hr: () => <Divider />,
        }}
      >
        {content}
      </ReactMarkdown>
    </div>
  )
}
