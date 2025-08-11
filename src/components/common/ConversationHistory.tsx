import React, { useEffect, useState } from 'react'
import { createPortal } from 'react-dom'
import {
  App,
  Button,
  Card,
  Empty,
  Flex,
  Input,
  Popconfirm,
  Typography,
} from 'antd'
import { DeleteOutlined, SearchOutlined } from '@ant-design/icons'
import { useTranslation } from 'react-i18next'
import { useNavigate, useParams } from 'react-router-dom'
import { useChatHistoryStore } from '../../store'
import { ConversationSummaryCard } from './ConversationSummaryCard'

const { Text } = Typography

interface ConversationHistoryProps {
  getSearchBoxContainer?: () => HTMLElement | null
}

export const ConversationHistory: React.FC<ConversationHistoryProps> = ({
  getSearchBoxContainer,
}) => {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const navigate = useNavigate()
  const { projectId } = useParams<{ projectId: string }>()
  const [, forceRender] = useState({})

  // Force a second render when getSearchBoxContainer is provided to ensure container is available
  useEffect(() => {
    if (getSearchBoxContainer) {
      forceRender({})
    }
  }, [])

  // Chat history store
  const {
    conversations,
    searchResults,
    isSearching,
    loading,
    loadingMore,
    deleting,
    error,
    listHasMore,
    searchHasMore,
    listTotal,
    searchTotal,
    selectedConversations,
    clearError,
    searchConversations,
    clearSearchResults,
    deleteConversationById,
    loadNextListPage,
    loadNextSearchPage,
    toggleConversationSelection,
    deselectAllConversations,
    deleteSelectedConversations,
  } = useChatHistoryStore(projectId)

  const [searchText, setSearchText] = useState('')

  // Show errors
  useEffect(() => {
    if (error) {
      message.error(error)
      clearError()
    }
  }, [error, message])

  useEffect(() => {
    if (searchText.trim()) {
      const timeoutId = setTimeout(() => {
        searchConversations(searchText)
      }, 500) // Debounce search for 500ms

      return () => clearTimeout(timeoutId)
    } else {
      clearSearchResults()
    }
  }, [searchText])

  const handleDeleteConversation = async (conversationId: string) => {
    return deleteConversationById(conversationId)
  }

  const handleLoadMore = async () => {
    try {
      if (searchText.trim()) {
        await loadNextSearchPage()
      } else {
        await loadNextListPage()
      }
    } catch (error) {
      // Error is handled by the store
      console.error('Failed to load more conversations:', error)
    }
  }

  const handleConversationSelect = (
    conversationId: string,
    _selected: boolean,
  ) => {
    toggleConversationSelection(conversationId)
  }

  const handleDeselectAll = () => {
    deselectAllConversations()
  }

  const handleDeleteSelected = async () => {
    try {
      await deleteSelectedConversations()
      message.success(
        `${selectedConversations.size} conversations deleted successfully`,
      )
    } catch (error) {
      // Error is handled by the store
      console.error('Failed to delete selected conversations:', error)
    }
  }

  // Calculate pagination info
  const getCurrentList = () =>
    searchText.trim() ? searchResults : conversations
  const getHasMore = () => (searchText.trim() ? searchHasMore : listHasMore)
  const getTotal = () => (searchText.trim() ? searchTotal : listTotal)

  // Check if we're in selection mode
  const isInSelectionMode = selectedConversations.size > 0

  // Search box component
  const searchBox = (
    <Input
      placeholder={t('forms.searchConversations')}
      allowClear
      size="middle"
      prefix={<SearchOutlined />}
      onChange={e => setSearchText(e.target.value)}
      className="self-center w-full"
    />
  )

  return (
    <>
      {/* Render search box in portal if container provided */}
      {getSearchBoxContainer &&
        (() => {
          const container = getSearchBoxContainer()
          return container ? createPortal(searchBox, container) : null
        })()}

      <div className="w-full h-full flex flex-col gap-3 overflow-y-hidden overflow-x-visible flex-1">
        {/* Search box - render inline if no container provided */}
        {!getSearchBoxContainer && (
          <div className="flex justify-end items-center w-full">
            {searchBox}
          </div>
        )}

        {/* Bulk actions bar */}
        {selectedConversations.size > 0 && (
          <div className="max-w-4xl w-full self-center px-3 pt-3">
            <Card
              classNames={{
                body: '!p-3',
              }}
            >
              <Flex
                justify="space-between"
                align="center"
                className={'flex-wrap gap-2'}
              >
                <Text strong>
                  {selectedConversations.size} conversation
                  {selectedConversations.size > 1 ? 's' : ''} selected
                </Text>
                <Flex className={'gap-2'}>
                  <Button onClick={handleDeselectAll}>Deselect All</Button>
                  <Popconfirm
                    title="Delete selected conversations"
                    description={`Are you sure you want to delete ${selectedConversations.size} conversation${selectedConversations.size > 1 ? 's' : ''}?`}
                    onConfirm={handleDeleteSelected}
                    okText="Yes"
                    cancelText="No"
                    okType="danger"
                    okButtonProps={{ loading: deleting }}
                  >
                    <Button danger icon={<DeleteOutlined />} loading={deleting}>
                      Delete Selected
                    </Button>
                  </Popconfirm>
                </Flex>
              </Flex>
            </Card>
          </div>
        )}

        {/* Conversation list */}
        <Flex className="flex-1 w-full flex-col !py-3 overflow-y-auto overflow-x-visible">
          <div
            className={'gap-2 max-w-4xl w-full self-center overflow-x-visible'}
          >
            {(searchText.trim() ? searchResults : conversations).length === 0 &&
            !loading &&
            !isSearching ? (
              <Card className={'!mx-3'}>
                <Empty
                  image={Empty.PRESENTED_IMAGE_SIMPLE}
                  description={
                    searchText.trim()
                      ? 'No conversations found matching your search'
                      : 'No chat history yet'
                  }
                >
                  {!searchText.trim() && !projectId && (
                    <Button type="primary" onClick={() => navigate('/')}>
                      Start New Chat
                    </Button>
                  )}
                </Empty>
              </Card>
            ) : (
              <div className="space-y-3 overflow-y-auto overflow-x-visible">
                {loading || isSearching ? (
                  <div className="flex justify-center py-8">
                    <div className="animate-spin rounded-full h-8 w-8 border-b-2"></div>
                  </div>
                ) : (
                  <Flex className="flex-col gap-3 w-full flex-1 overflow-y-auto overflow-x-visible">
                    {(searchText.trim() ? searchResults : conversations).map(
                      conversation => (
                        <div key={conversation.id} className="px-3">
                          <ConversationSummaryCard
                            conversation={conversation}
                            onDelete={handleDeleteConversation}
                            isSelected={selectedConversations.has(
                              conversation.id,
                            )}
                            onSelect={handleConversationSelect}
                            isInSelectionMode={isInSelectionMode}
                          />
                        </div>
                      ),
                    )}

                    {/* Pagination info */}
                    {getCurrentList().length > 0 && (
                      <Card
                        className="text-center !mx-3"
                        classNames={{
                          body: '!p-2 gap-2 flex justify-center items-center flex-wrap',
                        }}
                      >
                        <Text type="secondary">
                          Showing {getCurrentList().length} of {getTotal()}{' '}
                          conversations
                        </Text>
                        {getHasMore() && (
                          <Button
                            onClick={handleLoadMore}
                            loading={loadingMore}
                          >
                            Load More
                          </Button>
                        )}
                      </Card>
                    )}
                  </Flex>
                )}
              </div>
            )}
          </div>
        </Flex>
      </div>
    </>
  )
}
