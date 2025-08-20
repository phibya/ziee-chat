import React from 'react'
import {
  App,
  Button,
  Card,
  Checkbox,
  Divider,
  Popconfirm,
  theme,
  Tooltip,
  Typography,
} from 'antd'
import { DeleteOutlined } from '@ant-design/icons'
import { useTranslation } from 'react-i18next'
import { useLocation, useNavigate } from 'react-router-dom'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import { ConversationSummary, Permission } from '../../types'
import { setPreviousConversationListPagePath } from '../../store/ui/navigate.ts'
import { PermissionGuard } from '../Auth/PermissionGuard.tsx'

// Configure dayjs
dayjs.extend(relativeTime)

const { Text } = Typography

interface ConversationSummaryCardProps {
  conversation: ConversationSummary
  onDelete: (conversationId: string) => Promise<void>
  isSelected?: boolean
  onSelect?: (conversationId: string, selected: boolean) => void
  isInSelectionMode?: boolean
}

export const ConversationSummaryCard: React.FC<
  ConversationSummaryCardProps
> = ({
  conversation,
  onDelete,
  isSelected = false,
  onSelect,
  isInSelectionMode = false,
}) => {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const navigate = useNavigate()
  const { token } = theme.useToken()
  const location = useLocation()

  const handleCardClick = () => {
    if (isInSelectionMode && onSelect) {
      // In selection mode, toggle selection instead of navigating
      onSelect(conversation.id, !isSelected)
    } else {
      setPreviousConversationListPagePath(location.pathname)
      // Normal mode, navigate to conversation
      navigate(`/conversation/${conversation.id}`)
    }
  }

  const handleDeleteConversation = async () => {
    try {
      await onDelete(conversation.id)
      message.success(t('conversations.conversationDeleted'))
    } catch (error) {
      console.error('Failed to delete conversation:', error)
      // Error handling is done by the store, so we don't need to show another error message
    }
  }

  const handleSelectChange = (e: any) => {
    e.domEvent?.stopPropagation()
    if (onSelect) {
      onSelect(conversation.id, e.target.checked)
    }
  }

  return (
    <Card
      key={conversation.id}
      onClick={handleCardClick}
      className="cursor-pointer relative group hover:!shadow-md transition-shadow"
      classNames={{
        body: '!px-3 !py-2 flex gap-2 flex-col',
      }}
      hoverable
      style={{
        borderColor: isSelected ? token.colorPrimary : undefined,
      }}
    >
      <div className="flex items-start justify-between flex-wrap">
        <Text strong className="text-base" ellipsis={{ tooltip: true }}>
          {conversation.title}
        </Text>
        <div className="flex items-center gap-x-1 gap-y-0 flex-wrap">
          {conversation.message_count > 0 && (
            <>
              <Text type="secondary" className="text-xs font-normal">
                {conversation.message_count} messages
              </Text>
              <Divider type={'vertical'} />
            </>
          )}
          <Text type="secondary" className="whitespace-nowrap font-normal">
            {dayjs(conversation.updated_at).fromNow()}
          </Text>
        </div>
      </div>

      <div>
        <Text type="secondary" ellipsis={{ tooltip: false }}>
          {conversation.last_message || 'No messages yet'}
        </Text>
      </div>

      <PermissionGuard permissions={[Permission.ChatDelete]}>
        {/* Selection checkbox - positioned in bottom right */}
        {onSelect && (
          <div
            className={`absolute bottom-2 right-2 z-10 transition-opacity ${
              isSelected ? 'opacity-100' : 'opacity-0 group-hover:opacity-100'
            }`}
          >
            <Checkbox
              checked={isSelected}
              onChange={handleSelectChange}
              onClick={e => e.stopPropagation()}
            />
          </div>
        )}

        {!isInSelectionMode && (
          <Popconfirm
            title={t('conversations.deleteConversation')}
            description={t('history.deleteConfirm')}
            onConfirm={handleDeleteConversation}
            okText="Yes"
            cancelText="No"
            okButtonProps={{ loading: false }}
          >
            <Tooltip title={t('buttons.delete')}>
              <Button
                className="!absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity p-1 rounded"
                style={{
                  backgroundColor: token.colorBgContainer,
                }}
                onClick={(e: React.MouseEvent) => e.stopPropagation()}
              >
                <DeleteOutlined />
              </Button>
            </Tooltip>
          </Popconfirm>
        )}
      </PermissionGuard>
    </Card>
  )
}
