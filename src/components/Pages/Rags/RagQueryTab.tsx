import {
  App,
  Button,
  Card,
  Divider,
  Flex,
  Form,
  Input,
  Result,
  Space,
  Statistic,
  Tabs,
  Tag,
  Typography,
} from 'antd'
import React, { useEffect, useState } from 'react'
import { useParams } from 'react-router-dom'
import {
  clearQueryError,
  clearQueryResults,
  queryRAGInstance,
  Stores,
} from '../../../store'
import { generateFileDownloadToken } from '../../../store/files'
import { formatFileSize } from '../../../utils/fileUtils'
import {
  FileTextOutlined,
  SearchOutlined,
  BarChartOutlined,
  DownloadOutlined,
} from '@ant-design/icons'
import type {
  RAGQueryRequest,
  RAGSource,
  File as ApiFile,
} from '../../../types/api'

const { TextArea } = Input
const { Text, Paragraph } = Typography

interface QueryFormData extends RAGQueryRequest {
  query: string
}

export const RagQueryTab: React.FC = () => {
  const [form] = Form.useForm<QueryFormData>()
  const { message, modal } = App.useApp()
  const { ragInstanceId } = useParams<{ ragInstanceId: string }>()

  // Store usage - accessing query state from RAG store
  const { queryResults, querying, queryError, lastQuery } = Stores.RAG

  // Form submission handler
  const handleSubmit = async (values: QueryFormData) => {
    if (!ragInstanceId) {
      message.error('RAG instance ID not found')
      return
    }

    try {
      await queryRAGInstance(ragInstanceId, values)
      message.success('Query completed successfully')
    } catch (error) {
      console.error('Query failed:', error)
      // Error message will be shown by useEffect below
    }
  }

  // Handle errors from store
  useEffect(() => {
    if (queryError) {
      message.error(queryError)
      clearQueryError()
    }
  }, [queryError, message])

  // Handle form reset
  const handleReset = () => {
    form.resetFields()
    clearQueryResults()
  }

  // Format similarity score as percentage
  const formatSimilarity = (score: number): string => {
    return `${Math.round(score * 100)}%`
  }

  // File lookup helper
  const getFileInfo = (fileId: string): ApiFile | undefined => {
    return queryResults?.files.find(file => file.id === fileId)
  }

  // File modal content component (similar to FileCard.tsx)
  const FileModalContent: React.FC<{ file: ApiFile }> = ({ file }) => {
    const [downloadToken, setDownloadToken] = useState<string | null>(null)

    useEffect(() => {
      generateFileDownloadToken(file.id)
        .then(({ token }) => {
          setDownloadToken(token)
        })
        .catch(error => {
          console.error('Failed to generate download token:', error)
          message.error('Failed to generate download link')
        })
    }, [file.id])

    return (
      <div className="flex flex-col items-center gap-4 py-4">
        <div className="text-center">
          <div className="text-6xl mb-4">📄</div>
          <div className="pt-4">
            <Text type="secondary">
              File size: {formatFileSize(file.file_size)}
            </Text>
          </div>
        </div>
        <Button type="primary">
          <div className="flex justify-center">
            <a
              href={`/api/files/${file.id}/download-with-token?token=${downloadToken}`}
              download={file.filename}
              className="ant-btn ant-btn-primary"
            >
              <Typography.Text>
                <DownloadOutlined /> Download File
              </Typography.Text>
            </a>
          </div>
        </Button>
      </div>
    )
  }

  // Handle file name click to show modal
  const handleFileNameClick = (file: ApiFile) => {
    modal.info({
      icon: null,
      title: file.filename,
      width: 600,
      content: <FileModalContent file={file} />,
      footer: null,
      closable: true,
      maskClosable: true,
    })
  }

  // Truncate content with show more functionality
  const TruncatedContent: React.FC<{ content: string; maxLength?: number }> = ({
    content,
    maxLength = 300,
  }) => {
    const [expanded, setExpanded] = useState(false)
    const shouldTruncate = content.length > maxLength
    const displayContent =
      expanded || !shouldTruncate
        ? content
        : content.substring(0, maxLength) + '...'

    return (
      <div>
        <Paragraph className="mb-2">
          {displayContent}
          {shouldTruncate && (
            <Button
              type="link"
              size="small"
              onClick={() => setExpanded(!expanded)}
              className="p-0 h-auto ml-1"
            >
              {expanded ? 'Show less' : 'Show more'}
            </Button>
          )}
        </Paragraph>
      </div>
    )
  }

  // Results overview component
  const ResultsOverview: React.FC = () => {
    if (!queryResults) return null

    const { results, token_usage, metadata } = queryResults

    return (
      <Card size="small" className="mb-4">
        <Flex wrap gap="large" className="w-full">
          <Statistic
            title="Results Found"
            value={results.length}
            prefix={<SearchOutlined />}
            className="flex-1"
          />
          <Statistic
            title="Processing Time"
            value={metadata.processing_time_ms}
            suffix="ms"
            prefix={<BarChartOutlined />}
            className="flex-1"
          />
          <Statistic
            title="Chunks Retrieved"
            value={metadata.chunks_retrieved}
            prefix={<FileTextOutlined />}
            className="flex-1"
          />
          <Statistic
            title="Files Found"
            value={queryResults.files.length}
            prefix={<FileTextOutlined />}
            className="flex-1"
          />
          <Statistic
            title="Total Tokens"
            value={token_usage.total_tokens}
            className="flex-1"
          />
          {metadata.rerank_applied && <Tag color="blue">Reranked</Tag>}
        </Flex>
      </Card>
    )
  }

  // Source card component
  const SourceCard: React.FC<{ source: RAGSource; index: number }> = ({
    source,
    index,
  }) => {
    const { document, similarity_score } = source

    return (
      <Card
        size="small"
        title={
          <Flex justify="space-between" align="center">
            <Text strong>#{index + 1}</Text>
            <Text type="secondary" className="text-sm font-normal">
              Similarity: {formatSimilarity(similarity_score)}
            </Text>
          </Flex>
        }
        className="mb-3"
      >
        <TruncatedContent content={document.content} />

        <div className="mt-3 pt-3">
          <Space direction="vertical" size="small" className="w-full">
            <div className="flex items-center gap-2 text-xs">
              <Text type="secondary" className="text-xs">
                File:{' '}
                {getFileInfo(document.file_id) ? (
                  <Button
                    type="link"
                    size="small"
                    onClick={() =>
                      handleFileNameClick(getFileInfo(document.file_id)!)
                    }
                    className="p-0 h-auto text-xs"
                  >
                    {getFileInfo(document.file_id)!.filename}
                  </Button>
                ) : (
                  'Unknown'
                )}
              </Text>
              <Divider type="vertical" className="h-3" />
              <Text type="secondary" className="text-xs">
                Size:{' '}
                {getFileInfo(document.file_id)
                  ? formatFileSize(getFileInfo(document.file_id)!.file_size)
                  : 'Unknown'}
              </Text>
              <Divider type="vertical" className="h-3" />
              <Text type="secondary" className="text-xs">
                Chunk: {document.chunk_index}
              </Text>
            </div>
          </Space>
        </div>
      </Card>
    )
  }

  // File card component for Files tab
  const FileCard: React.FC<{ file: ApiFile }> = ({ file }) => {
    const handleDirectDownload = async (file: ApiFile) => {
      try {
        const { token } = await generateFileDownloadToken(file.id)
        const downloadUrl = `/api/files/${file.id}/download-with-token?token=${token}`
        const link = document.createElement('a')
        link.href = downloadUrl
        link.download = file.filename
        document.body.appendChild(link)
        link.click()
        document.body.removeChild(link)
      } catch (error) {
        console.error('Failed to generate download token:', error)
        message.error('Failed to download file')
      }
    }

    // Count chunks belonging to this file
    const chunkCount =
      queryResults?.results.filter(
        result => result.document.file_id === file.id,
      ).length || 0

    return (
      <Card size="small" className="mb-3">
        <Flex justify="space-between" align="center">
          <div>
            <Text>{file.filename}</Text>
            <br />
            <div className="flex items-center gap-2">
              <Text type="secondary" className="text-sm">
                Size: {formatFileSize(file.file_size)}
              </Text>
              <Divider type="vertical" className="h-3" />
              <Text type="secondary" className="text-sm">
                {chunkCount} chunk{chunkCount !== 1 ? 's' : ''}
              </Text>
            </div>
          </div>
          <Button
            type="primary"
            size="small"
            icon={<DownloadOutlined />}
            onClick={() => handleDirectDownload(file)}
          >
            Download
          </Button>
        </Flex>
      </Card>
    )
  }

  // Results section component
  const ResultsSection: React.FC = () => {
    if (!queryResults) return null

    const { results, files } = queryResults

    if (results.length === 0) {
      return (
        <Card>
          <Result
            icon={<SearchOutlined />}
            title="No Results Found"
            subTitle={`No relevant documents found for query: "${lastQuery}"`}
          />
        </Card>
      )
    }

    const tabItems = [
      {
        key: 'chunks',
        label: `Chunks (${results.length})`,
        children: (
          <Space direction="vertical" size="middle" className="w-full">
            {results.map((source: RAGSource, index: number) => (
              <SourceCard
                key={`${source.document.id}-${index}`}
                source={source}
                index={index}
              />
            ))}
          </Space>
        ),
      },
      {
        key: 'files',
        label: `Files (${files.length})`,
        children: (
          <Space direction="vertical" size="small" className="w-full">
            {files.map((file: ApiFile) => (
              <FileCard key={file.id} file={file} />
            ))}
          </Space>
        ),
      },
    ]

    return (
      <div>
        <ResultsOverview />
        <Tabs items={tabItems} defaultActiveKey="chunks" />
      </div>
    )
  }

  return (
    <Space direction="vertical" size="large" className="w-full">
      {/* Query Interface Card */}
      <Card title="Query Interface">
        <Form form={form} layout="vertical" onFinish={handleSubmit}>
          <Form.Item
            name="query"
            label="Query"
            rules={[
              { required: true, message: 'Please enter your query' },
              { min: 3, message: 'Query must be at least 3 characters long' },
            ]}
          >
            <TextArea
              placeholder="Enter your question or search query..."
              autoSize={{ minRows: 3, maxRows: 6 }}
              disabled={querying}
            />
          </Form.Item>

          {/* Submit buttons */}
          <Form.Item>
            <Space>
              <Button
                type="primary"
                htmlType="submit"
                loading={querying}
                icon={<SearchOutlined />}
              >
                {querying ? 'Querying...' : 'Query'}
              </Button>
              <Button onClick={handleReset} disabled={querying}>
                Clear
              </Button>
            </Space>
          </Form.Item>
        </Form>
      </Card>

      {/* Results Section in Separate Card */}
      {queryResults && (
        <Card title="Query Results">
          <ResultsSection />
        </Card>
      )}
    </Space>
  )
}
