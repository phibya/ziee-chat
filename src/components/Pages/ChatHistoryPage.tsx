import React, { useState, useEffect } from 'react'
import {
  Card,
  Table,
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

const { Title, Text } = Typography
const { Search } = Input

export const ChatHistoryPage: React.FC = () => {
  const { message } = App.useApp()
  const navigate = useNavigate()
  const [conversations, setConversations] = useState<ConversationSummary[]>([])
  const [filteredConversations, setFilteredConversations] = useState<
    ConversationSummary[]
  >([])
  const [loading, setLoading] = useState(false)
  const [searchText, setSearchText] = useState('')

  useEffect(() => {
    fetchConversations()
  }, [])

  useEffect(() => {
    if (searchText.trim()) {
      const filtered = conversations.filter(
        conversation =>
          conversation.title.toLowerCase().includes(searchText.toLowerCase()) ||
          conversation.last_message
            ?.toLowerCase()
            .includes(searchText.toLowerCase()),
      )
      setFilteredConversations(filtered)
    } else {
      setFilteredConversations(conversations)
    }
  }, [searchText, conversations])

  const fetchConversations = async () => {
    try {
      setLoading(true)
      const response = await ApiClient.Chat.listConversations({
        page: 1,
        per_page: 100,
      })
      setConversations(response.conversations)
      setFilteredConversations(response.conversations)
    } catch (error) {
      message.error('Failed to fetch chat history')
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
      // Delete all conversations one by one
      await Promise.all(
        conversations.map(conversation =>
          ApiClient.Chat.deleteConversation({
            conversation_id: conversation.id,
          }),
        ),
      )
      message.success('All chat history cleared successfully')
      fetchConversations()
    } catch (error) {
      message.error('Failed to clear chat history')
    }
  }

  const handleViewConversation = (conversation: ConversationSummary) => {
    // Navigate to chat page with this conversation
    navigate(`/?conversation=${conversation.id}`)
  }

  const columns = [
    {
      title: 'Title',
      dataIndex: 'title',
      key: 'title',
      render: (text: string, record: ConversationSummary) => (
        <Space>
          <MessageOutlined />
          <Text strong>{text}</Text>
          {record.message_count > 0 && (
            <Tag color="blue">{record.message_count} messages</Tag>
          )}
        </Space>
      ),
    },
    {
      title: 'Last Message',
      dataIndex: 'last_message',
      key: 'last_message',
      render: (text: string) => (
        <Text type="secondary" ellipsis={{ tooltip: true }}>
          {text || 'No messages'}
        </Text>
      ),
      ellipsis: true,
    },
    {
      title: 'Created At',
      dataIndex: 'created_at',
      key: 'created_at',
      render: (date: string) => new Date(date).toLocaleDateString(),
      sorter: (a: ConversationSummary, b: ConversationSummary) =>
        new Date(a.created_at).getTime() - new Date(b.created_at).getTime(),
      defaultSortOrder: 'descend' as const,
    },
    {
      title: 'Updated At',
      dataIndex: 'updated_at',
      key: 'updated_at',
      render: (date: string) => new Date(date).toLocaleDateString(),
      sorter: (a: ConversationSummary, b: ConversationSummary) =>
        new Date(a.updated_at).getTime() - new Date(b.updated_at).getTime(),
    },
    {
      title: 'Actions',
      key: 'actions',
      render: (_: any, record: ConversationSummary) => (
        <Space>
          <Tooltip title="View Conversation">
            <Button
              type="text"
              icon={<EyeOutlined />}
              onClick={() => handleViewConversation(record)}
            />
          </Tooltip>
          <Popconfirm
            title="Delete Conversation"
            description="Are you sure you want to delete this conversation?"
            onConfirm={() => handleDeleteConversation(record.id)}
            okText="Yes"
            cancelText="No"
          >
            <Tooltip title="Delete">
              <Button type="text" danger icon={<DeleteOutlined />} />
            </Tooltip>
          </Popconfirm>
        </Space>
      ),
    },
  ]

  return (
    <div className="p-6">
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

          <Card>
            {filteredConversations.length === 0 && !loading ? (
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
            ) : (
              <Table
                columns={columns}
                dataSource={filteredConversations}
                loading={loading}
                rowKey="id"
                pagination={{
                  pageSize: 20,
                  showSizeChanger: true,
                  showQuickJumper: true,
                  showTotal: (total, range) =>
                    `${range[0]}-${range[1]} of ${total} conversations`,
                }}
              />
            )}
          </Card>
        </Col>
      </Row>
    </div>
  )
}
