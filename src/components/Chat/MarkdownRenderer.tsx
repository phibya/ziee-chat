import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import remarkBreaks from 'remark-breaks'
import rehypeHighlight from 'rehype-highlight'
import rehypeRaw from 'rehype-raw'
import { Typography, Divider } from 'antd'
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
          // Custom components using Ant Design theme and Tailwind
          p: ({ children }) => (
            <Text className="block mb-3 leading-relaxed">
              {children}
            </Text>
          ),
          code: ({ inline, className, children, ...props }: any) => {
            if (inline) {
              return (
                <code
                  className="px-2 py-1 rounded text-sm font-mono"
                  {...props}
                >
                  {children}
                </code>
              )
            }
            return (
              <pre className="p-4 rounded-lg overflow-auto my-3">
                <code className={className} {...props}>
                  {children}
                </code>
              </pre>
            )
          },
          blockquote: ({ children }) => (
            <blockquote className="border-l-4 pl-4 my-4 italic py-2 pr-4 rounded-r-lg">
              <Text type="secondary">{children}</Text>
            </blockquote>
          ),
          h1: ({ children }) => (
            <Title level={2} className="mt-6 mb-4">
              {children}
            </Title>
          ),
          h2: ({ children }) => (
            <Title level={3} className="mt-5 mb-3">
              {children}
            </Title>
          ),
          h3: ({ children }) => (
            <Title level={4} className="mt-4 mb-2">
              {children}
            </Title>
          ),
          ul: ({ children }) => (
            <ul className="pl-6 my-2 space-y-1">
              {children}
            </ul>
          ),
          ol: ({ children }) => (
            <ol className="pl-6 my-2 space-y-1">
              {children}
            </ol>
          ),
          li: ({ children }) => (
            <li className="leading-relaxed">
              <Text>{children}</Text>
            </li>
          ),
          strong: ({ children }) => (
            <Text strong>{children}</Text>
          ),
          em: ({ children }) => (
            <Text italic>{children}</Text>
          ),
          a: ({ href, children }) => (
            <Link href={href} target="_blank" rel="noopener noreferrer">
              {children}
            </Link>
          ),
          table: ({ children }) => (
            <div className="overflow-x-auto my-4">
              <table className="w-full border-collapse">
                {children}
              </table>
            </div>
          ),
          th: ({ children }) => (
            <th className="p-3 text-left">
              <Text strong>{children}</Text>
            </th>
          ),
          td: ({ children }) => (
            <td className="p-3">
              <Text>{children}</Text>
            </td>
          ),
          hr: () => (
            <Divider className="my-6" />
          ),
        }}
      >
        {content}
      </ReactMarkdown>
    </div>
  )
}