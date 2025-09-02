import React, { useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useNavigate } from 'react-router-dom'
import { ConversationHistory } from '../../common/ConversationHistory.tsx'
import { Button, Typography } from 'antd'
import { TauriDragRegion } from '../../common/TauriDragRegion.tsx'
import { TitleBarWrapper } from '../../common/TitleBarWrapper.tsx'
import { useMainContentMinSize } from '../../hooks/useWindowMinSize.ts'
import { PermissionGuard } from '../../Auth/PermissionGuard.tsx'
import { Permission } from '../../../types'
import { SearchOutlined, MessageOutlined, PlusOutlined } from '@ant-design/icons'
import { useChatHistoryStore } from '../../../store'

const { Title, Text } = Typography

export const ChatHistoryPage: React.FC = () => {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const searchBoxContainerRef = useRef<HTMLDivElement>(null)
  const pageMinSize = useMainContentMinSize()
  const [isSearchBoxVisible, setIsSearchBoxVisible] = useState(false)
  
  // Chat history store for empty state detection
  const { conversations, loading } = useChatHistoryStore()

  return (
    <div className="h-full w-full flex flex-col overflow-y-hidden">
      <TitleBarWrapper>
        <div className="h-full flex items-center justify-between w-full ">
          <TauriDragRegion className={'h-full w-full absolute top-0 left-0'} />
          <Typography.Title level={4} className="!m-0 !leading-tight">
            {t('pages.chatHistory')}
          </Typography.Title>
          <PermissionGuard permissions={[Permission.ChatSearch]}>
            <div className="h-full flex items-center justify-between">
              {pageMinSize.xs ? (
                <Button
                  type={isSearchBoxVisible ? 'primary' : 'text'}
                  icon={<SearchOutlined />}
                  style={{
                    fontSize: '18px',
                  }}
                  onClick={() => setIsSearchBoxVisible(!isSearchBoxVisible)}
                />
              ) : (
                <div ref={searchBoxContainerRef} />
              )}
            </div>
          </PermissionGuard>
        </div>
      </TitleBarWrapper>
      <div className="flex-1 flex flex-col overflow-hidden items-center">
        {pageMinSize.xs && isSearchBoxVisible && (
          <div className={'w-full max-w-96 px-3 pt-3'}>
            <div ref={searchBoxContainerRef} />
          </div>
        )}
        
        {/* Show ConversationHistory if there are conversations or loading */}
        {(conversations.length > 0 || loading) && (
          <div className="flex flex-1 flex-col w-full justify-center overflow-hidden">
            <div className={'h-full flex flex-col overflow-y-auto'}>
              <ConversationHistory
                key={pageMinSize.xs + ''}
                getSearchBoxContainer={() => searchBoxContainerRef.current}
              />
            </div>
          </div>
        )}

        {/* Empty State - similar to RagsPage */}
        {!loading && conversations.length === 0 && (
          <div className="text-center py-12 m-auto">
            <MessageOutlined className="text-6xl mb-4" />
            <Title level={3} type="secondary">
              No chat history yet
            </Title>
            <Text type="secondary" className="block mb-4">
              Start your first conversation to see your chat history here
            </Text>
            <Button
              type="primary"
              icon={<PlusOutlined />}
              onClick={() => navigate('/')}
            >
              Start New Chat
            </Button>
          </div>
        )}
      </div>
    </div>
  )
}
