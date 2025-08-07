import { App, Button, Input, Modal, Typography } from 'antd'
import { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { useNavigate } from 'react-router-dom'
import {
  DeleteOutlined,
  EditOutlined,
  ExclamationCircleOutlined,
  MessageOutlined,
} from '@ant-design/icons'
import {
  loadAllRecentConversations,
  removeConversationFromList,
  updateExistingConversation,
  Stores,
} from '../../../store'
import { ConversationSummary } from '../../../types/api/chat'

const { confirm } = Modal
const { Text } = Typography

export function RecentConversations() {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const { message } = App.useApp()
  const [editingId, setEditingId] = useState<string | null>(null)
  const [editingTitle, setEditingTitle] = useState('')

  const { conversations, isLoading } = Stores.Conversations

  useEffect(() => {
    // Only load if we don't have conversations yet
    if (conversations.length === 0) {
      loadAllRecentConversations()
    }
  }, [conversations.length])

  const handleConversationClick = (conversationId: string) => {
    navigate(`/conversation/${conversationId}`)
  }

  const handleEditTitle = (conversation: ConversationSummary) => {
    setEditingId(conversation.id)
    setEditingTitle(conversation.title)
  }

  const handleSaveTitle = async () => {
    if (!editingId || !editingTitle.trim()) return

    try {
      await updateExistingConversation(editingId, {
        title: editingTitle.trim(),
      })

      setEditingId(null)
      setEditingTitle('')
      message.success(t('conversations.renamed') || 'Renamed successfully')
    } catch (error: any) {
      console.error('Failed to update conversation:', error)
      message.error(
        error?.message || t('common.failedToRename') || 'Failed to rename',
      )
    }
  }

  const handleCancelEdit = () => {
    setEditingId(null)
    setEditingTitle('')
  }

  const handleDeleteConversation = (conversation: ConversationSummary) => {
    confirm({
      title: t('conversations.deleteTitle') || 'Delete Conversation',
      icon: <ExclamationCircleOutlined />,
      content:
        t('conversations.deleteConfirm', { title: conversation.title }) ||
        `Are you sure you want to delete "${conversation.title}"?`,
      okText: t('common.delete') || 'Delete',
      okType: 'danger',
      cancelText: t('common.cancel') || 'Cancel',
      onOk: async () => {
        try {
          await removeConversationFromList(conversation.id)
          message.success(t('conversations.deleted') || 'Conversation deleted')
        } catch (error: any) {
          console.error('Failed to delete conversation:', error)
          message.error(
            error?.message || t('common.failedToDelete') || 'Failed to delete',
          )
        }
      },
    })
  }

  return (
    <div className="flex-1 overflow-y-auto space-y-0">
      {isLoading ? (
        <div className="text-center p-3 text-gray-500">
          <div>{t('common.loading') || 'Loading...'}</div>
        </div>
      ) : conversations.length === 0 ? (
        <div className="text-center p-3 text-gray-500">
          <MessageOutlined className="mb-2" />
          <div>{t('conversations.noConversations') || 'No conversations'}</div>
        </div>
      ) : (
        conversations.map((conversation: ConversationSummary) => (
          <div key={conversation.id} className="group relative">
            {editingId === conversation.id ? (
              <div className="flex items-center px-3 py-1 mx-2">
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
              <div
                className="flex items-center px-3 py-1 mx-2 rounded-md cursor-pointer transition-colors duration-150 text-gray-700 hover:bg-gray-200"
                onClick={() => handleConversationClick(conversation.id)}
              >
                <Text className="flex-1 truncate">{conversation.title}</Text>

                {/* Action buttons - only visible on hover */}
                <div className="opacity-0 group-hover:opacity-100 flex ml-2">
                  <Button
                    size="small"
                    type="text"
                    icon={<EditOutlined />}
                    onClick={e => {
                      e.stopPropagation()
                      handleEditTitle(conversation)
                    }}
                    style={{
                      width: '20px',
                      height: '20px',
                      minWidth: '20px',
                      padding: '0',
                      fontSize: '10px',
                    }}
                  />
                  <Button
                    size="small"
                    type="text"
                    icon={<DeleteOutlined />}
                    onClick={e => {
                      e.stopPropagation()
                      handleDeleteConversation(conversation)
                    }}
                    style={{
                      width: '20px',
                      height: '20px',
                      minWidth: '20px',
                      padding: '0',
                      fontSize: '10px',
                      color: '#ff4d4f',
                    }}
                  />
                </div>
              </div>
            )}
          </div>
        ))
      )}
    </div>
  )
}
