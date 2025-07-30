import { useEffect } from 'react'
import { App } from 'antd'
import { useTranslation } from 'react-i18next'
import { useNavigate } from 'react-router-dom'
import { ChatInput } from './ChatInput'
import {
  Stores,
  clearChatError,
  loadUserAssistants,
  loadAllModelProviders,
} from '../../store'

export function NewChatInterface() {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const navigate = useNavigate()

  // Chat store
  const { error: chatError } = Stores.Chat

  // Assistants store
  const { loading: assistantsLoading } = Stores.Assistants

  // Model providers store
  const { loading: providersLoading } = Stores.Providers


  useEffect(() => {
    initializeData()
  }, [])

  // Show errors
  useEffect(() => {
    if (chatError) {
      message.error(chatError)
      clearChatError()
    }
  }, [chatError]) // Removed message and clearChatError from dependencies to prevent infinite rerenders

  const initializeData = async () => {
    try {
      await Promise.all([loadUserAssistants(), loadAllModelProviders()])
    } catch (error: any) {
      message.error(error?.message || 'Failed to load data')
    }
  }

  const handleNewConversationCreated = (id: string) => {
    navigate(`/conversation/${id}`)
  }




  if (assistantsLoading || providersLoading) {
    return <div>Loading...</div>
  }

  return (
    <div className="flex flex-col h-full">
      {/* Welcome message */}
      <div className="flex flex-col items-center justify-center flex-1 text-center p-8">
        <div className="mb-8">
          <div className="text-3xl font-light mb-4">
            {t('chat.placeholderWelcome')}
          </div>
        </div>

        <div className="w-full max-w-2xl">
          <ChatInput onNewConversationCreated={handleNewConversationCreated} />
        </div>
      </div>
    </div>
  )
}
