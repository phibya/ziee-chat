import { useEffect, useState } from 'react'
import { App } from 'antd'
import { useTranslation } from 'react-i18next'
import { useNavigate } from 'react-router-dom'
import { useShallow } from 'zustand/react/shallow'
import { ChatWelcome } from './ChatWelcome'
import { useChatStore } from '../../store/chat'
import { useAssistantsStore } from '../../store/assistants'
import { useModelProvidersStore } from '../../store/modelProviders'
import { useConversationsStore } from '../../store'
import { useAuthStore } from '../../store/auth'

export function NewChatInterface() {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const navigate = useNavigate()

  // Auth store
  const { user } = useAuthStore()

  // Chat store
  const {
    createConversation,
    sendMessage,
    error: chatError,
    clearError: clearChatError,
  } = useChatStore(
    useShallow(state => ({
      createConversation: state.createConversation,
      sendMessage: state.sendMessage,
      error: state.error,
      clearError: state.clearError,
    })),
  )

  // Assistants store
  const {
    assistants,
    loading: assistantsLoading,
    loadAssistants,
  } = useAssistantsStore(
    useShallow(state => ({
      assistants: state.assistants,
      loading: state.loading,
      loadAssistants: state.loadAssistants,
    })),
  )

  // Model providers store
  const {
    providers: modelProviders,
    loading: providersLoading,
    loadProviders,
  } = useModelProvidersStore(
    useShallow(state => ({
      providers: state.providers,
      loading: state.loading,
      loadProviders: state.loadProviders,
    })),
  )

  // Conversations store
  const { addConversation } = useConversationsStore()

  const [selectedAssistant, setSelectedAssistant] = useState<string | null>(
    null,
  )
  const [selectedModel, setSelectedModel] = useState<string | null>(null)

  useEffect(() => {
    initializeData()
  }, [])

  // Show errors
  useEffect(() => {
    if (chatError) {
      message.error(chatError)
      clearChatError()
    }
  }, [chatError, message, clearChatError])

  const initializeData = async () => {
    try {
      await Promise.all([loadAssistants(), loadProviders()])
    } catch (error: any) {
      message.error(error?.message || 'Failed to load data')
    }
  }

  // Set default selections when data loads
  useEffect(() => {
    if (assistants.length > 0 && !selectedAssistant) {
      // Filter to only show user's own assistants in chat (not admin templates)
      const userAssistants = assistants.filter(a => !a.is_template)
      if (userAssistants.length > 0) {
        setSelectedAssistant(userAssistants[0].id)
      }
    }
  }, [assistants, selectedAssistant])

  useEffect(() => {
    if (modelProviders.length > 0 && !selectedModel) {
      // Filter to only show enabled providers
      const enabledProviders = modelProviders.filter(p => p.enabled)
      if (enabledProviders.length > 0) {
        const firstProvider = enabledProviders[0]
        if (firstProvider && firstProvider.models.length > 0) {
          const enabledModels = firstProvider.models.filter(m => m.enabled)
          if (enabledModels.length > 0) {
            setSelectedModel(`${firstProvider.id}:${enabledModels[0].id}`)
          }
        }
      }
    }
  }, [modelProviders, selectedModel])

  const createNewConversation = async () => {
    if (!selectedAssistant || !selectedModel) {
      message.error(t('chat.noAssistantSelected'))
      return null
    }

    const [providerId, modelId] = selectedModel.split(':')

    try {
      const conversationId = await createConversation(
        selectedAssistant,
        providerId,
        modelId,
      )

      // Add to conversations store immediately
      addConversation({
        id: conversationId,
        title: 'New Conversation',
        user_id: user?.id || '',
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
        last_message: undefined,
        message_count: 0,
      })

      // Navigate to the new conversation URL
      navigate(`/conversation/${conversationId}`)
      return conversationId
    } catch (error) {
      // Error is already handled by the store
      console.error('Failed to create conversation:', error)
      return null
    }
  }

  const handleSend = async (inputValue: string) => {
    if (!inputValue.trim() || !selectedAssistant || !selectedModel) return

    // Create new conversation and send first message
    const conversationId = await createNewConversation()
    if (!conversationId) return

    const [providerId, modelId] = selectedModel.split(':')

    try {
      // Send the first message
      await sendMessage(
        inputValue.trim(),
        selectedAssistant,
        providerId,
        modelId,
      )
    } catch (error) {
      // Error is already handled by the store
      console.error('Failed to send first message:', error)
    }
  }

  if (assistantsLoading || providersLoading) {
    return <div>Loading...</div>
  }

  // Filter to only show user's own assistants in chat (not admin templates)
  const userAssistants = assistants.filter(a => !a.is_template)

  // Filter to only show enabled providers
  const enabledProviders = modelProviders.filter(p => p.enabled)

  return (
    <ChatWelcome
      selectedAssistant={selectedAssistant}
      selectedModel={selectedModel}
      assistants={userAssistants}
      modelProviders={enabledProviders}
      onAssistantChange={setSelectedAssistant}
      onModelChange={setSelectedModel}
      onSend={handleSend}
    />
  )
}
