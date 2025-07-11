import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { App, Button, Input, Modal, Tooltip, Typography } from 'antd'
import {
  DeleteOutlined,
  EditOutlined,
  ExclamationCircleOutlined,
  MessageOutlined,
} from '@ant-design/icons'
import { useTranslation } from 'react-i18next'
import { ConversationSummary } from '../../types/api/chat'
import { useConversationsStore } from '../../store'
import { useShallow } from 'zustand/react/shallow'

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
  const { t } = useTranslation()
  const navigate = useNavigate()
  const { message } = App.useApp()
  const [editingId, setEditingId] = useState<string | null>(null)
  const [editingTitle, setEditingTitle] = useState('')

  const {
    conversations,
    isLoading,
    loadConversations,
    updateConversation,
    removeConversation,
  } = useConversationsStore(
    useShallow(state => ({
      conversations: state.conversations,
      isLoading: state.isLoading,
      loadConversations: state.loadConversations,
      updateConversation: state.updateConversation,
      removeConversation: state.removeConversation,
    })),
  )

  const isExpanded = isMobile ? mobileOverlayOpen : !collapsed

  useEffect(() => {
    // Only load if we don't have conversations yet
    if (conversations.length === 0) {
      loadConversations()
    }
  }, [])

  const handleConversationClick = (conversationId: string) => {
    navigate(`/conversation/${conversationId}`)
    onConversationClick?.()
  }

  const handleEditTitle = (conversation: ConversationSummary) => {
    setEditingId(conversation.id)
    setEditingTitle(conversation.title)
  }

  const handleSaveTitle = async () => {
    if (!editingId || !editingTitle.trim()) return

    try {
      // Use store method that handles API call
      await updateConversation(editingId, { title: editingTitle.trim() })

      setEditingId(null)
      setEditingTitle('')
      message.success(t('conversations.renamed'))
    } catch (error: any) {
      console.error('Failed to update conversation:', error)
      message.error(error?.message || t('common.failedToRename'))
    }
  }

  const handleCancelEdit = () => {
    setEditingId(null)
    setEditingTitle('')
  }

  const handleDeleteConversation = (conversation: ConversationSummary) => {
    confirm({
      title: t('conversations.deleteTitle'),
      icon: <ExclamationCircleOutlined />,
      content: t('conversations.deleteConfirm', { title: conversation.title }),
      okText: t('common.delete'),
      okType: 'danger',
      cancelText: t('common.cancel'),
      onOk: async () => {
        try {
          // Use store method that handles API call
          await removeConversation(conversation.id)

          message.success(t('conversations.deleted'))
        } catch (error: any) {
          console.error('Failed to delete conversation:', error)
          message.error(error?.message || t('common.failedToDelete'))
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
              className="cursor-pointer"
            />
          </Tooltip>
        ))}
      </div>
    )
  }

  return (
    <div className="flex-1 overflow-auto max-w-42 pl-2">
      {isLoading ? (
        <div className="text-center">
          <div>{t('common.loading')}</div>
        </div>
      ) : conversations.length === 0 ? (
        <div className="text-center">
          <MessageOutlined />
          <div>{t('conversations.noConversations')}</div>
        </div>
      ) : (
        conversations.map(conversation => (
          <div key={conversation.id} className="group relative">
            {editingId === conversation.id ? (
              <div className="flex items-center">
                <Input
                  value={editingTitle}
                  onChange={e => setEditingTitle(e.target.value)}
                  onPressEnter={handleSaveTitle}
                  onBlur={handleSaveTitle}
                  autoFocus
                  size="small"
                />
                <Button size="small" type="text" onClick={handleCancelEdit}>
                  ×
                </Button>
              </div>
            ) : (
              <>
                <Typography.Text
                  ellipsis
                  onClick={() => handleConversationClick(conversation.id)}
                >
                  {conversation.title}
                </Typography.Text>

                {/* Last message preview */}
                {conversation.last_message && (
                  <Typography.Text type="secondary" ellipsis>
                    {conversation.last_message.substring(0, 50)}
                    {conversation.last_message.length > 50 ? '...' : ''}
                  </Typography.Text>
                )}

                {/* Action buttons - only visible on hover */}
                <div className="absolute right-2 top-1/2 transform -translate-y-1/2 opacity-0 group-hover:opacity-100 flex">
                  <Tooltip title={t('conversations.rename')}>
                    <Button
                      size="small"
                      type="text"
                      icon={<EditOutlined />}
                      onClick={e => {
                        e.stopPropagation()
                        handleEditTitle(conversation)
                      }}
                    />
                  </Tooltip>
                  <Tooltip title={t('conversations.delete')}>
                    <Button
                      size="small"
                      type="text"
                      icon={<DeleteOutlined />}
                      onClick={e => {
                        e.stopPropagation()
                        handleDeleteConversation(conversation)
                      }}
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
