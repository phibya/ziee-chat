import React, { useState, useEffect } from 'react'
import {
  Card,
  Button,
  Input,
  Space,
  Popconfirm,
  Typography,
  Tag,
  Tooltip,
  Row,
  Col,
  Empty,
} from 'antd'
import {
  DeleteOutlined,
  MessageOutlined,
  SearchOutlined,
  ClearOutlined,
  EyeOutlined,
} from '@ant-design/icons'
import { useNavigate } from 'react-router-dom'
import { ApiClient } from '../../api/client'
import { ConversationSummary } from '../../types/api/chat'
import { App } from 'antd'
import { PageContainer } from '../common/PageContainer'

const { Title, Text } = Typography
const { Search } = Input

export const ChatHistoryPage: React.FC = () => {
  const { message } = App.useApp()
  const navigate = useNavigate()
  const [conversations, setConversations] = useState<ConversationSummary[]>([])
  const [loading, setLoading] = useState(false)
  const [searchText, setSearchText] = useState('')

  useEffect(() => {
    fetchConversations()
  }, [])

  useEffect(() => {
    if (searchText.trim()) {
      const timeoutId = setTimeout(() => {
        searchConversations(searchText)
      }, 500) // Debounce search for 500ms

      return () => clearTimeout(timeoutId)
    } else {
      fetchConversations()
    }
  }, [searchText])

  const fetchConversations = async () => {
    try {
      setLoading(true)
      const response = await ApiClient.Chat.listConversations({
        page: 1,
        per_page: 100,
      })
      setConversations(response.conversations)
    } catch (error) {
      message.error('Failed to fetch chat history')
    } finally {
      setLoading(false)
    }
  }

  const searchConversations = async (query: string) => {
    try {
      setLoading(true)
      const response = await ApiClient.Chat.searchConversations({
        q: query,
        page: 1,
        per_page: 100,
      })
      setConversations(response.conversations)
    } catch (error) {
      message.error('Failed to search conversations')
    } finally {
      setLoading(false)
    }
  }

  const handleDeleteConversation = async (conversationId: string) => {
    try {
      await ApiClient.Chat.deleteConversation({
        conversation_id: conversationId,
      })
      message.success('Conversation deleted successfully')
      fetchConversations()
    } catch (error) {
      message.error('Failed to delete conversation')
    }
  }

  const handleClearAllHistory = async () => {
    try {
      const response = await ApiClient.Chat.clearAllConversations()
      message.success(
        `${response.deleted_count} conversations deleted successfully`,
      )
      fetchConversations()
    } catch (error) {
      message.error('Failed to clear chat history')
    }
  }

  const handleViewConversation = (conversation: ConversationSummary) => {
    // Navigate to chat page with this conversation
    navigate(`/conversation/${conversation.id}`)
  }

  const formatDate = (dateStr: string) => {
    const date = new Date(dateStr)
    const now = new Date()
    const diffTime = Math.abs(now.getTime() - date.getTime())
    const diffDays = Math.floor(diffTime / (1000 * 60 * 60 * 24))

    if (diffDays === 0) {
      return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
    } else if (diffDays === 1) {
      return 'Yesterday'
    } else if (diffDays < 7) {
      return `${diffDays} days ago`
    } else {
      return date.toLocaleDateString()
    }
  }

  const renderConversationCard = (conversation: ConversationSummary) => (
    <Card
      key={conversation.id}
      className="mb-4 hover:shadow-md transition-shadow cursor-pointer"
      onClick={() => handleViewConversation(conversation)}
      hoverable
      actions={[
        <Tooltip title="View Conversation" key="view">
          <EyeOutlined
            onClick={e => {
              e.stopPropagation()
              handleViewConversation(conversation)
            }}
          />
        </Tooltip>,
        <Popconfirm
          key="delete"
          title="Delete Conversation"
          description="Are you sure you want to delete this conversation?"
          onConfirm={() => handleDeleteConversation(conversation.id)}
          okText="Yes"
          cancelText="No"
        >
          <Tooltip title="Delete">
            <DeleteOutlined
              className=""
              onClick={(e: React.MouseEvent) => e.stopPropagation()}
            />
          </Tooltip>
        </Popconfirm>,
      ]}
    >
      <Card.Meta
        avatar={<MessageOutlined className="text-lg" />}
        title={
          <div className="flex items-center justify-between">
            <Text strong className="text-base" ellipsis={{ tooltip: true }}>
              {conversation.title}
            </Text>
            <div className="flex items-center gap-2 ml-2">
              {conversation.message_count > 0 && (
                <Tag color="blue" className="text-xs">
                  {conversation.message_count} messages
                </Tag>
              )}
              <Text type="secondary" className="text-xs whitespace-nowrap">
                {formatDate(conversation.updated_at)}
              </Text>
            </div>
          </div>
        }
        description={
          <div className="mt-2">
            <Text type="secondary" ellipsis={{ tooltip: true }}>
              {conversation.last_message || 'No messages yet'}
            </Text>
          </div>
        }
      />
    </Card>
  )

  return (
    <PageContainer>
        <Row gutter={[24, 24]}>
          <Col span={24}>
            <div className="flex justify-between items-center mb-6">
              <div>
                <Title level={2}>Chat History</Title>
                <Text type="secondary">
                  View and manage all your chat conversations
                </Text>
              </div>
              <Space>
                <Search
                  placeholder="Search conversations..."
                  allowClear
                  enterButton={<SearchOutlined />}
                  size="middle"
                  onSearch={setSearchText}
                  onChange={e => setSearchText(e.target.value)}
                  style={{ width: 300 }}
                />
                {conversations.length > 0 && (
                  <Popconfirm
                    title="Clear All History"
                    description="Are you sure you want to delete all chat history? This action cannot be undone."
                    onConfirm={handleClearAllHistory}
                    okText="Yes"
                    cancelText="No"
                    okType="danger"
                  >
                    <Button danger icon={<ClearOutlined />} type="default">
                      Clear All
                    </Button>
                  </Popconfirm>
                )}
              </Space>
            </div>

            {conversations.length === 0 && !loading ? (
              <Card>
                <Empty
                  image={Empty.PRESENTED_IMAGE_SIMPLE}
                  description={
                    searchText.trim()
                      ? 'No conversations found matching your search'
                      : 'No chat history yet'
                  }
                >
                  {!searchText.trim() && (
                    <Button type="primary" onClick={() => navigate('/')}>
                      Start New Chat
                    </Button>
                  )}
                </Empty>
              </Card>
            ) : (
              <div className="space-y-4">
                {loading ? (
                  <div className="flex justify-center py-8">
                    <div className="animate-spin rounded-full h-8 w-8 border-b-2"></div>
                  </div>
                ) : (
                  <>
                    {conversations.map(renderConversationCard)}
                    {conversations.length > 20 && (
                      <Card className="text-center">
                        <Text type="secondary">
                          Showing {Math.min(20, conversations.length)} of{' '}
                          {conversations.length} conversations
                        </Text>
                      </Card>
                    )}
                  </>
                )}
              </div>
            )}
          </Col>
        </Row>
    </PageContainer>
  )
}
