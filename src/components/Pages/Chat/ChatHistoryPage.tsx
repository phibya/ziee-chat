import React, { useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { ConversationHistory } from '../common/ConversationHistory'
import { Button, Typography } from 'antd'
import { TauriDragRegion } from '../common/TauriDragRegion.tsx'
import { TitleBarWrapper } from '../common/TitleBarWrapper.tsx'
import { useMainContentMinSize } from '../hooks/useWindowMinSize.ts'
import { PermissionGuard } from '../Auth/PermissionGuard.tsx'
import { Permission } from '../../types'
import { SearchOutlined } from '@ant-design/icons'

export const ChatHistoryPage: React.FC = () => {
  const { t } = useTranslation()
  const searchBoxContainerRef = useRef<HTMLDivElement>(null)
  const pageMinSize = useMainContentMinSize()
  const [isSearchBoxVisible, setIsSearchBoxVisible] = useState(false)

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
      <div className="w-full flex-1 flex flex-col overflow-y-hidden">
        {pageMinSize.xs ? (
          <div
            className={'w-full flex items-center justify-center px-3 pt-3'}
            style={{
              display: isSearchBoxVisible ? 'flex' : 'none',
            }}
          >
            <div className={'w-full max-w-96'} ref={searchBoxContainerRef} />
          </div>
        ) : null}
        <div
          className={
            'w-full flex flex-1 items-center justify-center overflow-y-auto'
          }
        >
          <ConversationHistory
            key={pageMinSize.xs + ''}
            getSearchBoxContainer={() => searchBoxContainerRef.current}
          />
        </div>
      </div>
    </div>
  )
}
