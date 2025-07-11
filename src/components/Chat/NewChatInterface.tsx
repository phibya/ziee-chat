import { useEffect, useState } from 'react'
import { App } from 'antd'
import { useTranslation } from 'react-i18next'
import { useNavigate } from 'react-router-dom'
import { ApiClient } from '../../api/client'
import { CreateConversationRequest } from '../../types/api/chat'
import { Assistant } from '../../types/api/assistant'
import { ModelProvider } from '../../types/api/modelProvider'
import { User } from '../../types/api/user'
import { ChatWelcome } from './ChatWelcome'
import { useConversationsStore } from '../../store'

export function NewChatInterface() {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const navigate = useNavigate()
  const { addConversation } = useConversationsStore()

  const [assistants, setAssistants] = useState<Assistant[]>([])
  const [modelProviders, setModelProviders] = useState<ModelProvider[]>([])
  const [_currentUser, setCurrentUser] = useState<User | null>(null)
  const [selectedAssistant, setSelectedAssistant] = useState<string | null>(
    null,
  )
  const [selectedModel, setSelectedModel] = useState<string | null>(null)

  useEffect(() => {
    initializeData()
  }, [])

  const initializeData = async () => {
    try {
      const [assistantsResponse, providersResponse, userResponse] =
        await Promise.all([
          ApiClient.Assistant.list({ page: 1, per_page: 100 }),
          ApiClient.ModelProviders.list({ page: 1, per_page: 100 }),
          ApiClient.Auth.me(),
        ])

      // Filter to only show user's own assistants in chat (not template ones)
      const userAssistants = assistantsResponse.assistants.filter(
        a => !a.is_template,
      )
      setAssistants(userAssistants)
      setCurrentUser(userResponse)

      // The backend already filters model providers based on permissions
      const availableProviders = providersResponse.providers.filter(
        p => p.enabled,
      )
      setModelProviders(availableProviders)

      // Set default selections
      if (userAssistants.length > 0) {
        setSelectedAssistant(userAssistants[0].id)
      }
      if (availableProviders.length > 0) {
        const firstProvider = availableProviders[0]
        if (firstProvider && firstProvider.models.length > 0) {
          setSelectedModel(`${firstProvider.id}:${firstProvider.models[0].id}`)
        }
      }
    } catch (error) {
      message.error('Failed to load data')
    }
  }

  const createNewConversation = async () => {
    if (!selectedAssistant || !selectedModel) {
      message.error(t('chat.noAssistantSelected'))
      return null
    }

    const [providerId, modelId] = selectedModel.split(':')

    try {
      const request: CreateConversationRequest = {
        title: 'New Conversation',
        assistant_id: selectedAssistant,
        model_provider_id: providerId,
        model_id: modelId,
      }

      const newConversation = await ApiClient.Chat.createConversation(request)

      // Add to store immediately with basic info
      addConversation({
        id: newConversation.id,
        title: newConversation.title,
        user_id: newConversation.user_id,
        assistant_id: newConversation.assistant_id,
        model_provider_id: newConversation.model_provider_id,
        model_id: newConversation.model_id,
        created_at: newConversation.created_at,
        updated_at: newConversation.updated_at,
        last_message: undefined,
        message_count: 0,
      })

      // Navigate to the new conversation URL
      navigate(`/conversation/${newConversation.id}`)
      return newConversation
    } catch (error) {
      message.error('Failed to create conversation')
      return null
    }
  }

  const handleSend = async (inputValue: string) => {
    if (!inputValue.trim() || !selectedAssistant || !selectedModel) return

    // Create new conversation and send first message
    const conversation = await createNewConversation()
    if (!conversation) return

    const [providerId, modelId] = selectedModel.split(':')

    try {
      // Send the first message
      await ApiClient.Chat.sendMessage({
        conversation_id: conversation.id,
        content: inputValue.trim(),
        model_provider_id: providerId,
        model_id: modelId,
      })
    } catch (error) {
      console.error('Failed to send first message:', error)
      message.error('Failed to send message')
    }
  }

  return (
    <ChatWelcome
      selectedAssistant={selectedAssistant}
      selectedModel={selectedModel}
      assistants={assistants}
      modelProviders={modelProviders}
      onAssistantChange={setSelectedAssistant}
      onModelChange={setSelectedModel}
      onSend={handleSend}
    />
  )
}
