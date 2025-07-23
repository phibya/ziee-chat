import React, { useEffect, useState } from 'react'
import {
  App,
  Button,
  Card,
  Col,
  Empty,
  Input,
  Popconfirm,
  Row,
  Flex,
  Tag,
  Tooltip,
  Typography,
} from 'antd'
import {
  ClearOutlined,
  DeleteOutlined,
  EyeOutlined,
  MessageOutlined,
  SearchOutlined,
} from '@ant-design/icons'
import { useNavigate } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { ConversationSummary } from '../../types/api/chat'
import { PageContainer } from '../common/PageContainer'
import {
  Stores,
  loadChatHistoryConversationsList,
  searchChatHistoryConversations,
  deleteChatHistoryConversationById,
  clearAllUserChatHistoryConversations,
  clearChatHistorySearchResults,
  clearChatHistoryStoreError,
} from '../../store'

const { Title, Text } = Typography
const { Search } = Input

export const ChatHistoryPage: React.FC = () => {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const navigate = useNavigate()

  // Chat history store
  const {
    conversations,
    searchResults,
    isSearching,
    loading,
    deleting,
    clearing,
    error,
  } = Stores.ChatHistory

  const [searchText, setSearchText] = useState('')

  useEffect(() => {
    loadChatHistoryConversationsList()
  }, [])

  // Show errors
  useEffect(() => {
    if (error) {
      message.error(error)
      clearChatHistoryStoreError()
    }
  }, [error, message])

  useEffect(() => {
    if (searchText.trim()) {
      const timeoutId = setTimeout(() => {
        searchChatHistoryConversations(searchText)
      }, 500) // Debounce search for 500ms

      return () => clearTimeout(timeoutId)
    } else {
      clearChatHistorySearchResults()
      if (conversations.length === 0) {
        loadChatHistoryConversationsList()
      }
    }
  }, [searchText, conversations.length])

  const handleDeleteConversation = async (conversationId: string) => {
    try {
      await deleteChatHistoryConversationById(conversationId)
      message.success(t('conversations.conversationDeleted'))
    } catch (error) {
      // Error is handled by the store
      console.error('Failed to delete conversation:', error)
    }
  }

  const handleClearAllHistory = async () => {
    try {
      await clearAllUserChatHistoryConversations()
      message.success(t('conversations.allConversationsDeleted'))
    } catch (error) {
      // Error is handled by the store
      console.error('Failed to clear chat history:', error)
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
        <Tooltip title={t('conversations.viewConversation')} key="view">
          <EyeOutlined
            onClick={e => {
              e.stopPropagation()
              handleViewConversation(conversation)
            }}
          />
        </Tooltip>,
        <Popconfirm
          key="delete"
          title={t('conversations.deleteConversation')}
          description={t('history.deleteConfirm')}
          onConfirm={() => handleDeleteConversation(conversation.id)}
          okText="Yes"
          cancelText="No"
          okButtonProps={{ loading: deleting }}
        >
          <Tooltip title={t('buttons.delete')}>
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
              <Title level={2}>{t('pages.chatHistory')}</Title>
              <Text type="secondary">
                View and manage all your chat conversations
              </Text>
            </div>
            <Flex className="gap-2">
              <Search
                placeholder={t('forms.searchConversations')}
                allowClear
                enterButton={<SearchOutlined />}
                size="middle"
                onSearch={setSearchText}
                onChange={e => setSearchText(e.target.value)}
                style={{ width: 300 }}
              />
              {conversations.length > 0 && (
                <Popconfirm
                  title={t('conversations.clearAllHistory')}
                  description={t('history.clearAllConfirm')}
                  onConfirm={handleClearAllHistory}
                  okText="Yes"
                  cancelText="No"
                  okType="danger"
                  okButtonProps={{ loading: clearing }}
                >
                  <Button
                    danger
                    icon={<ClearOutlined />}
                    type="default"
                    loading={clearing}
                  >
                    Clear All
                  </Button>
                </Popconfirm>
              )}
            </Flex>
          </div>

          {(searchText.trim() ? searchResults : conversations).length === 0 &&
          !loading &&
          !isSearching ? (
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
              {loading || isSearching ? (
                <div className="flex justify-center py-8">
                  <div className="animate-spin rounded-full h-8 w-8 border-b-2"></div>
                </div>
              ) : (
                <>
                  {(searchText.trim() ? searchResults : conversations).map(
                    renderConversationCard,
                  )}
                  {(searchText.trim() ? searchResults : conversations).length >
                    20 && (
                    <Card className="text-center">
                      <Text type="secondary">
                        Showing{' '}
                        {Math.min(
                          20,
                          (searchText.trim() ? searchResults : conversations)
                            .length,
                        )}{' '}
                        of{' '}
                        {
                          (searchText.trim() ? searchResults : conversations)
                            .length
                        }{' '}
                        conversations
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
