import { App, Button, theme, Typography } from 'antd'
import { useTranslation } from 'react-i18next'
import { useNavigate } from 'react-router-dom'
import {
  DeleteOutlined,
  ExclamationCircleOutlined,
  MessageOutlined,
} from '@ant-design/icons'
import {
  removeConversationFromList,
  setSidebarCollapsed,
  Stores,
} from '../../../store'
import { ConversationSummary } from '../../../types'
import { useWindowMinSize } from '../../hooks/useWindowMinSize.ts'

const { Text } = Typography

export function RecentConversations() {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const { message, modal } = App.useApp()
  const { token } = theme.useToken()

  const { conversations, isLoading } = Stores.Conversations
  const windowMinSize = useWindowMinSize()

  const handleConversationClick = (conversationId: string) => {
    navigate(`/conversation/${conversationId}`)
    windowMinSize.xs && setSidebarCollapsed(true)
  }

  const handleDeleteConversation = (conversation: ConversationSummary) => {
    modal.confirm({
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
          // Navigate to home if the deleted conversation was active
          const conversationId = window.location.pathname.split('/').pop()
          if (conversationId === conversation.id) {
            navigate('/')
          }
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
            <div
              className="flex items-center px-3 py-1 mx-2 rounded-md cursor-pointer transition-colors duration-150"
              onClick={() => handleConversationClick(conversation.id)}
              onMouseEnter={e => {
                e.currentTarget.style.backgroundColor = token.colorPrimaryHover
              }}
              onMouseLeave={e => {
                e.currentTarget.style.backgroundColor = 'transparent'
              }}
            >
              <Text className="flex-1 truncate">{conversation.title}</Text>

              {/* Action buttons - only visible on hover */}
              <div className="opacity-0 group-hover:opacity-100 flex absolute right-3">
                <Button
                  size="small"
                  type="text"
                  icon={<DeleteOutlined />}
                  onClick={e => {
                    e.stopPropagation()
                    handleDeleteConversation(conversation)
                  }}
                />
              </div>
            </div>
          </div>
        ))
      )}
    </div>
  )
}
