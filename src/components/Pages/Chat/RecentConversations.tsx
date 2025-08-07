import {
  DeleteOutlined,
  EditOutlined,
  ExclamationCircleOutlined,
  MessageOutlined,
} from '@ant-design/icons'
import { App, Button, Input, Modal, theme, Tooltip, Typography } from 'antd'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Link, useNavigate } from 'react-router-dom'
import {
  closeMobileOverlay,
  loadAllRecentConversations,
  removeConversationFromList,
  Stores,
  updateExistingConversation,
} from '../../../store'
import { useUILeftPanelCollapsed } from '../../../store/settings.ts'
import { ConversationSummary } from '../../../types/api/chat'

const { confirm } = Modal

export function RecentConversations() {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const { message } = App.useApp()
  const [editingId, setEditingId] = useState<string | null>(null)
  const [editingTitle, setEditingTitle] = useState('')
  const { token } = theme.useToken()

  const leftPanelCollapsed = useUILeftPanelCollapsed()
  const { isMobile, mobileOverlayOpen } = Stores.UI.Layout

  const { conversations, isLoading } = Stores.Conversations

  const isExpanded = isMobile ? mobileOverlayOpen : !leftPanelCollapsed

  useEffect(() => {
    // Only load if we don't have conversations yet
    if (conversations.length === 0) {
      loadAllRecentConversations()
    }
  }, [])

  const handleConversationClick = (conversationId: string) => {
    navigate(`/conversation/${conversationId}`)
    if (isMobile) {
      closeMobileOverlay()
    }
  }

  const handleEditTitle = (conversation: ConversationSummary) => {
    setEditingId(conversation.id)
    setEditingTitle(conversation.title)
  }

  const handleSaveTitle = async () => {
    if (!editingId || !editingTitle.trim()) return

    try {
      // Use store method that handles API call
      await updateExistingConversation(editingId, {
        title: editingTitle.trim(),
      })

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
          await removeConversationFromList(conversation.id)

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
            <Link to={`/conversation/${conversation.id}`}>
              <div
                onClick={() => handleConversationClick(conversation.id)}
                className="cursor-pointer"
              />
            </Link>
          </Tooltip>
        ))}
      </div>
    )
  }

  return (
    <div className={'w-full h-full'}>
      <div className="flex-1 max-w-42 pl-2 flex flex-col gap-1">
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
            <div key={conversation.id} className="group relative !py-[0.5]">
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
                  <Link to={`/conversation/${conversation.id}`}>
                    <Typography.Text
                      ellipsis
                      onClick={() => handleConversationClick(conversation.id)}
                    >
                      {conversation.title}
                    </Typography.Text>
                  </Link>

                  {/* Action buttons - only visible on hover */}
                  <div
                    className="absolute right-2 top-1/2 transform -translate-y-1/2 opacity-0 group-hover:opacity-100 flex rounded"
                    style={{
                      backgroundColor: token.colorBgContainer,
                    }}
                  >
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
    </div>
  )
}
