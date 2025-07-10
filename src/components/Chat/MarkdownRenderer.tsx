import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import remarkBreaks from 'remark-breaks'
import rehypeHighlight from 'rehype-highlight'
import rehypeRaw from 'rehype-raw'
import { Typography } from 'antd'
import 'prismjs/themes/prism-tomorrow.css'

const { Text } = Typography

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
          // Custom components for better styling
          p: ({ children }) => (
            <Text
              style={{
                color: 'inherit',
                display: 'block',
                marginBottom: '0.75em',
                lineHeight: '1.6',
              }}
            >
              {children}
            </Text>
          ),
          code: ({ inline, className, children, ...props }: any) => {
            if (inline) {
              return (
                <code
                  style={{
                    backgroundColor: 'rgba(255, 255, 255, 0.1)',
                    padding: '2px 6px',
                    borderRadius: '4px',
                    fontSize: '0.875em',
                    fontFamily:
                      'ui-monospace, SFMono-Regular, "SF Mono", Monaco, Consolas, "Liberation Mono", "Courier New", monospace',
                    color: '#f8f8f2',
                  }}
                  {...props}
                >
                  {children}
                </code>
              )
            }
            return (
              <pre
                style={{
                  backgroundColor: 'rgba(0, 0, 0, 0.4)',
                  padding: '1rem',
                  borderRadius: '8px',
                  overflow: 'auto',
                  margin: '0.75em 0',
                  border: '1px solid rgba(255, 255, 255, 0.1)',
                }}
              >
                <code className={className} {...props}>
                  {children}
                </code>
              </pre>
            )
          },
          blockquote: ({ children }) => (
            <blockquote
              style={{
                borderLeft: '4px solid #f97316',
                paddingLeft: '1rem',
                margin: '1rem 0',
                fontStyle: 'italic',
                color: 'rgba(255, 255, 255, 0.8)',
                backgroundColor: 'rgba(249, 115, 22, 0.1)',
                padding: '0.5rem 1rem',
                borderRadius: '0 8px 8px 0',
              }}
            >
              {children}
            </blockquote>
          ),
          h1: ({ children }) => (
            <h1
              style={{
                color: 'rgba(255, 255, 255, 0.95)',
                marginTop: '1.5rem',
                marginBottom: '1rem',
                fontSize: '1.5rem',
                fontWeight: 600,
              }}
            >
              {children}
            </h1>
          ),
          h2: ({ children }) => (
            <h2
              style={{
                color: 'rgba(255, 255, 255, 0.95)',
                marginTop: '1.25rem',
                marginBottom: '0.75rem',
                fontSize: '1.25rem',
                fontWeight: 600,
              }}
            >
              {children}
            </h2>
          ),
          h3: ({ children }) => (
            <h3
              style={{
                color: 'rgba(255, 255, 255, 0.95)',
                marginTop: '1rem',
                marginBottom: '0.5rem',
                fontSize: '1.125rem',
                fontWeight: 600,
              }}
            >
              {children}
            </h3>
          ),
          ul: ({ children }) => (
            <ul
              style={{
                paddingLeft: '1.5rem',
                margin: '0.5rem 0',
              }}
            >
              {children}
            </ul>
          ),
          ol: ({ children }) => (
            <ol
              style={{
                paddingLeft: '1.5rem',
                margin: '0.5rem 0',
              }}
            >
              {children}
            </ol>
          ),
          li: ({ children }) => (
            <li
              style={{
                color: 'rgba(255, 255, 255, 0.9)',
                marginBottom: '0.25rem',
                lineHeight: '1.6',
              }}
            >
              {children}
            </li>
          ),
          strong: ({ children }) => (
            <strong
              style={{
                color: 'rgba(255, 255, 255, 0.95)',
                fontWeight: 600,
              }}
            >
              {children}
            </strong>
          ),
          em: ({ children }) => (
            <em
              style={{
                color: 'rgba(255, 255, 255, 0.9)',
                fontStyle: 'italic',
              }}
            >
              {children}
            </em>
          ),
          a: ({ href, children }) => (
            <a
              href={href}
              target="_blank"
              rel="noopener noreferrer"
              style={{
                color: '#f97316',
                textDecoration: 'underline',
              }}
            >
              {children}
            </a>
          ),
          table: ({ children }) => (
            <div style={{ overflowX: 'auto', margin: '1rem 0' }}>
              <table
                style={{
                  width: '100%',
                  borderCollapse: 'collapse',
                  border: '1px solid rgba(255, 255, 255, 0.2)',
                }}
              >
                {children}
              </table>
            </div>
          ),
          th: ({ children }) => (
            <th
              style={{
                border: '1px solid rgba(255, 255, 255, 0.2)',
                padding: '0.75rem',
                backgroundColor: 'rgba(255, 255, 255, 0.1)',
                color: 'rgba(255, 255, 255, 0.95)',
                fontWeight: 600,
                textAlign: 'left',
              }}
            >
              {children}
            </th>
          ),
          td: ({ children }) => (
            <td
              style={{
                border: '1px solid rgba(255, 255, 255, 0.2)',
                padding: '0.75rem',
                color: 'rgba(255, 255, 255, 0.9)',
              }}
            >
              {children}
            </td>
          ),
          hr: () => (
            <hr
              style={{
                border: 'none',
                borderTop: '1px solid rgba(255, 255, 255, 0.2)',
                margin: '1.5rem 0',
              }}
            />
          ),
        }}
      >
        {content}
      </ReactMarkdown>
    </div>
  )
}
