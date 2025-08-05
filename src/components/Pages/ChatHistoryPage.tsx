import React, { useRef } from 'react'
import { useTranslation } from 'react-i18next'
import { PageContainer } from '../common/PageContainer'
import { ConversationHistory } from '../common/ConversationHistory'

export const ChatHistoryPage: React.FC = () => {
  const { t } = useTranslation()
  const searchBoxContainerRef = useRef<HTMLDivElement>(null)

  return (
    <PageContainer
      title={t('pages.chatHistory')}
      extra={<div ref={searchBoxContainerRef} />}
    >
      <div className="w-full h-full flex flex-col gap-4 overflow-y-auto">
        <ConversationHistory
          getSearchBoxContainer={() => searchBoxContainerRef.current}
        />
      </div>
    </PageContainer>
  )
}
