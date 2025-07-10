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
                  className="bg-gray-100 dark:bg-gray-800 px-2 py-1 rounded text-sm font-mono"
                  {...props}
                >
                  {children}
                </code>
              )
            }
            return (
              <pre className="bg-gray-900 dark:bg-gray-800 p-4 rounded-lg overflow-auto my-3 border border-gray-200 dark:border-gray-700">
                <code className={className} {...props}>
                  {children}
                </code>
              </pre>
            )
          },
          blockquote: ({ children }) => (
            <blockquote className="border-l-4 border-primary pl-4 my-4 italic bg-primary bg-opacity-5 py-2 pr-4 rounded-r-lg">
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
              <table className="w-full border-collapse border border-gray-200 dark:border-gray-700">
                {children}
              </table>
            </div>
          ),
          th: ({ children }) => (
            <th className="border border-gray-200 dark:border-gray-700 p-3 bg-gray-50 dark:bg-gray-800 text-left">
              <Text strong>{children}</Text>
            </th>
          ),
          td: ({ children }) => (
            <td className="border border-gray-200 dark:border-gray-700 p-3">
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