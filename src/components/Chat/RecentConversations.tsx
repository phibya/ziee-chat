import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { Button, Tooltip, Modal, Input } from 'antd'
import {
  MessageOutlined,
  EditOutlined,
  DeleteOutlined,
  ExclamationCircleOutlined,
} from '@ant-design/icons'
import { useTheme } from '../../hooks/useTheme'
import { ApiClient } from '../../api/client'
import { ConversationSummary } from '../../types/api/chat'
import { App } from 'antd'

const { confirm } = Modal

interface RecentConversationsProps {
  collapsed?: boolean
  isMobile?: boolean
  mobileOverlayOpen?: boolean
  onConversationClick?: () => void
}

export function RecentConversations({
  collapsed = false,
  isMobile = false,
  mobileOverlayOpen = false,
  onConversationClick,
}: RecentConversationsProps) {
  const navigate = useNavigate()
  const appTheme = useTheme()
  const { message } = App.useApp()
  const [conversations, setConversations] = useState<ConversationSummary[]>([])
  const [loading, setLoading] = useState(false)
  const [editingId, setEditingId] = useState<string | null>(null)
  const [editingTitle, setEditingTitle] = useState('')

  const isExpanded = isMobile ? mobileOverlayOpen : !collapsed

  useEffect(() => {
    loadConversations()
  }, [])

  const loadConversations = async () => {
    setLoading(true)
    try {
      const response = await ApiClient.Chat.listConversations({
        page: 1,
        per_page: 20, // Show recent 20 conversations
      })
      setConversations(response.conversations)
    } catch (error) {
      console.error('Failed to load conversations:', error)
      message.error('Failed to load recent conversations')
    } finally {
      setLoading(false)
    }
  }

  const handleConversationClick = (conversationId: string) => {
    navigate(`/?conversation=${conversationId}`)
    onConversationClick?.()
  }

  const handleEditTitle = (conversation: ConversationSummary) => {
    setEditingId(conversation.id)
    setEditingTitle(conversation.title)
  }

  const handleSaveTitle = async () => {
    if (!editingId || !editingTitle.trim()) return

    try {
      await ApiClient.Chat.updateConversation({
        conversation_id: editingId,
        title: editingTitle.trim(),
      })
      
      // Update local state
      setConversations(prevConversations =>
        prevConversations.map(conv =>
          conv.id === editingId
            ? { ...conv, title: editingTitle.trim() }
            : conv
        )
      )
      
      setEditingId(null)
      setEditingTitle('')
      message.success('Conversation renamed successfully')
    } catch (error) {
      console.error('Failed to update conversation:', error)
      message.error('Failed to rename conversation')
    }
  }

  const handleCancelEdit = () => {
    setEditingId(null)
    setEditingTitle('')
  }

  const handleDeleteConversation = (conversation: ConversationSummary) => {
    confirm({
      title: 'Delete Conversation',
      icon: <ExclamationCircleOutlined />,
      content: `Are you sure you want to delete "${conversation.title}"? This action cannot be undone.`,
      okText: 'Delete',
      okType: 'danger',
      cancelText: 'Cancel',
      onOk: async () => {
        try {
          await ApiClient.Chat.deleteConversation({
            conversation_id: conversation.id,
          })
          
          // Remove from local state
          setConversations(prevConversations =>
            prevConversations.filter(conv => conv.id !== conversation.id)
          )
          
          message.success('Conversation deleted successfully')
        } catch (error) {
          console.error('Failed to delete conversation:', error)
          message.error('Failed to delete conversation')
        }
      },
    })
  }

  if (!isExpanded) {
    // Collapsed state - show dots for conversations
    return (
      <div className="flex-1 overflow-auto">
        {conversations.slice(0, 10).map(conversation => (
          <Tooltip
            key={conversation.id}
            title={conversation.title}
            placement="right"
            mouseEnterDelay={0.5}
          >
            <div
              onClick={() => handleConversationClick(conversation.id)}
              className="w-2 h-2 rounded-full mx-auto my-1.5 cursor-pointer transition-all duration-200"
              style={{
                backgroundColor: appTheme.sidebarTextSecondary,
              }}
              onMouseEnter={e => {
                e.currentTarget.style.backgroundColor = 'rgba(255,255,255,0.6)'
              }}
              onMouseLeave={e => {
                e.currentTarget.style.backgroundColor = appTheme.sidebarTextSecondary
              }}
            />
          </Tooltip>
        ))}
      </div>
    )
  }

  return (
    <div className="flex-1 overflow-auto">
      {loading ? (
        <div
          className="py-8 px-4 text-center"
          style={{ color: appTheme.sidebarTextSecondary }}
        >
          <div className="text-sm">Loading...</div>
        </div>
      ) : conversations.length === 0 ? (
        <div
          className="py-8 px-4 text-center"
          style={{ color: appTheme.sidebarTextSecondary }}
        >
          <MessageOutlined
            className="text-2xl mb-2"
            style={{ color: appTheme.sidebarTextSecondary }}
          />
          <div className="text-sm">No conversations yet</div>
        </div>
      ) : (
        conversations.map(conversation => (
          <div
            key={conversation.id}
            className="group px-3 py-2 mb-0.5 rounded-lg transition-all duration-200 border relative"
            style={{
              backgroundColor: 'transparent',
              color: appTheme.sidebarText,
              borderColor: 'transparent',
            }}
            onMouseEnter={e => {
              e.currentTarget.style.backgroundColor = 'rgba(255,255,255,0.05)'
            }}
            onMouseLeave={e => {
              e.currentTarget.style.backgroundColor = 'transparent'
            }}
          >
            {editingId === conversation.id ? (
              <div className="flex items-center gap-2">
                <Input
                  value={editingTitle}
                  onChange={e => setEditingTitle(e.target.value)}
                  onPressEnter={handleSaveTitle}
                  onBlur={handleSaveTitle}
                  autoFocus
                  size="small"
                  className="flex-1"
                />
                <Button
                  size="small"
                  type="text"
                  onClick={handleCancelEdit}
                  style={{ color: appTheme.sidebarTextSecondary }}
                >
                  ×
                </Button>
              </div>
            ) : (
              <>
                <div
                  onClick={() => handleConversationClick(conversation.id)}
                  className="cursor-pointer text-sm overflow-hidden text-ellipsis whitespace-nowrap pr-16"
                >
                  {conversation.title}
                </div>
                
                {/* Last message preview */}
                {conversation.last_message && (
                  <div
                    className="text-xs mt-1 opacity-60 overflow-hidden text-ellipsis whitespace-nowrap pr-16"
                    style={{ color: appTheme.sidebarTextSecondary }}
                  >
                    {conversation.last_message.substring(0, 50)}
                    {conversation.last_message.length > 50 ? '...' : ''}
                  </div>
                )}
                
                {/* Action buttons - only visible on hover */}
                <div className="absolute right-2 top-1/2 transform -translate-y-1/2 opacity-0 group-hover:opacity-100 transition-opacity duration-200 flex gap-1">
                  <Tooltip title="Rename">
                    <Button
                      size="small"
                      type="text"
                      icon={<EditOutlined />}
                      onClick={e => {
                        e.stopPropagation()
                        handleEditTitle(conversation)
                      }}
                      className="p-1"
                      style={{ color: appTheme.sidebarTextSecondary }}
                    />
                  </Tooltip>
                  <Tooltip title="Delete">
                    <Button
                      size="small"
                      type="text"
                      icon={<DeleteOutlined />}
                      onClick={e => {
                        e.stopPropagation()
                        handleDeleteConversation(conversation)
                      }}
                      className="p-1"
                      style={{ color: appTheme.sidebarTextSecondary }}
                    />
                  </Tooltip>
                </div>
              </>
            )}
          </div>
        ))
      )}
    </div>
  )
}