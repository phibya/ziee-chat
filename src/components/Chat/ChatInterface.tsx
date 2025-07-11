import { useEffect, useState } from 'react'
import { App, Flex } from 'antd'
import { useTranslation } from 'react-i18next'
import { useNavigate } from 'react-router-dom'
import { ApiClient } from '../../api/client'
import {
  Conversation,
  CreateConversationRequest,
  Message,
} from '../../types/api/chat'
import { Assistant } from '../../types/api/assistant'
import { ModelProvider } from '../../types/api/modelProvider'
import { User } from '../../types/api/user'
import { ChatHeader } from './ChatHeader'
import { ChatWelcome } from './ChatWelcome'
import { ChatMessageList } from './ChatMessageList'
import { ChatInput } from './ChatInput'

interface ChatInterfaceProps {
  conversationId: string | null
}

export function ChatInterface({ conversationId }: ChatInterfaceProps) {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const navigate = useNavigate()
  const [inputValue, setInputValue] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [conversation, setConversation] = useState<Conversation | null>(null)
  const [messages, setMessages] = useState<Message[]>([])
  const [isStreaming, setIsStreaming] = useState(false)
  const [assistants, setAssistants] = useState<Assistant[]>([])
  const [modelProviders, setModelProviders] = useState<ModelProvider[]>([])
  const [_currentUser, setCurrentUser] = useState<User | null>(null)
  const [selectedAssistant, setSelectedAssistant] = useState<string | null>(
    null,
  )
  const [selectedModel, setSelectedModel] = useState<string | null>(null)
  const [editingMessage, setEditingMessage] = useState<string | null>(null)
  const [editValue, setEditValue] = useState('')
  const [messageBranches, setMessageBranches] = useState<{
    [key: string]: Message[]
  }>({})
  const [loadingBranches, setLoadingBranches] = useState<{
    [key: string]: boolean
  }>({})

  const getAuthToken = () => {
    if (typeof window === 'undefined') return null
    // eslint-disable-next-line no-undef
    const authData = localStorage.getItem('auth-storage')
    if (authData) {
      try {
        const parsed = JSON.parse(authData)
        return parsed.state?.token || null
      } catch {
        return null
      }
    }
    return null
  }

  useEffect(() => {
    initializeData()
  }, [])

  useEffect(() => {
    if (conversationId) {
      loadConversation(conversationId)
    }
  }, [conversationId])

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
      // Admin users with config::model-providers::read permission get all providers
      // Other users only get providers assigned to their groups
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

  const loadConversation = async (conversationId: string) => {
    try {
      const conversationResponse = await ApiClient.Chat.getConversation({
        conversation_id: conversationId,
      })

      setConversation(conversationResponse.conversation)
      setMessages(conversationResponse.messages)

      if (conversationResponse.conversation.assistant_id) {
        setSelectedAssistant(conversationResponse.conversation.assistant_id)
      }
      if (conversationResponse.conversation.model_provider_id) {
        setSelectedModel(
          `${conversationResponse.conversation.model_provider_id}:${conversationResponse.conversation.model_id}`,
        )
      }
    } catch (error) {
      message.error('Failed to load conversation')
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
      setConversation(newConversation)
      setMessages([])
      // Navigate to the new conversation URL
      navigate(`/conversation/${newConversation.id}`)
      return newConversation
    } catch (error) {
      message.error('Failed to create conversation')
      return null
    }
  }

  const handleSend = async () => {
    if (!inputValue.trim() || !selectedAssistant || !selectedModel) return

    let currentConversation = conversation
    if (!currentConversation) {
      currentConversation = await createNewConversation()
      if (!currentConversation) return
    }

    const [providerId, modelId] = selectedModel.split(':')
    const userInput = inputValue.trim()
    setInputValue('')
    setIsLoading(true)
    setIsStreaming(true)

    // Add user message immediately to UI
    const tempUserMessage: Message = {
      id: `temp-${Date.now()}`,
      conversation_id: currentConversation.id,
      content: userInput,
      role: 'user',
      model_provider_id: providerId,
      model_id: modelId,
      branch_id: '', // Will be set by backend
      is_active_branch: true,
      originated_from_id: undefined,
      edit_count: 0,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    }

    // Add temp assistant message for streaming
    const tempAssistantMessage: Message = {
      id: `temp-assistant-${Date.now()}`,
      conversation_id: currentConversation.id,
      content: '',
      role: 'assistant',
      model_provider_id: providerId,
      model_id: modelId,
      branch_id: '',
      is_active_branch: true,
      originated_from_id: undefined,
      edit_count: 0,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    }

    setMessages(prev => [...prev, tempUserMessage, tempAssistantMessage])

    // Get auth token
    const token = getAuthToken()
    if (!token) {
      message.error('Authentication required')
      setIsLoading(false)
      setIsStreaming(false)
      setMessages(prev =>
        prev.filter(
          msg =>
            msg.id !== tempUserMessage.id && msg.id !== tempAssistantMessage.id,
        ),
      )
      return
    }

    try {
      // First try using the regular API client to test the endpoint
      await ApiClient.Chat.sendMessage({
        conversation_id: currentConversation.id,
        content: userInput,
        model_provider_id: providerId,
        model_id: modelId,
      })

      // If successful, reload the conversation to get the actual messages
      const conversationResponse = await ApiClient.Chat.getConversation({
        conversation_id: currentConversation.id,
      })

      setMessages(conversationResponse.messages)
      setIsLoading(false)
      setIsStreaming(false)
    } catch (error) {
      console.error('Chat error:', error)
      message.error(
        'Failed to send message: ' +
          (error instanceof Error ? error.message : 'Unknown error'),
      )
      setIsLoading(false)
      setIsStreaming(false)
      setMessages(prev =>
        prev.filter(
          msg =>
            msg.id !== tempUserMessage.id && msg.id !== tempAssistantMessage.id,
        ),
      )
    }
  }

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      handleSend()
    }
  }

  const handleStopGeneration = () => {
    setIsLoading(false)
    setIsStreaming(false)
    message.info('Generation stopped')
  }

  const handleEditMessage = (messageId: string, content: string) => {
    setEditingMessage(messageId)
    setEditValue(content)
  }

  const handleSaveEdit = async () => {
    if (!editingMessage || !editValue.trim()) return

    try {
      const originalMessage = messages.find(m => m.id === editingMessage)
      const contentChanged =
        originalMessage && originalMessage.content.trim() !== editValue.trim()

      await ApiClient.Chat.editMessage({
        message_id: editingMessage,
        content: editValue.trim(),
      })

      setEditingMessage(null)
      setEditValue('')

      // Immediately reload conversation to show the updated branch
      if (conversation) {
        const conversationResponse = await ApiClient.Chat.getConversation({
          conversation_id: conversation.id,
        })
        setMessages(conversationResponse.messages)
      }

      if (contentChanged) {
        message.success('Message updated and sent to AI for response')
      } else {
        message.success('Message updated successfully')
      }
    } catch (error) {
      message.error('Failed to update message')
    }
  }

  const handleCancelEdit = () => {
    setEditingMessage(null)
    setEditValue('')
  }

  const loadMessageBranches = async (msg: Message) => {
    if (!conversation) return

    const branchKey = `${msg.id}`

    setLoadingBranches(prev => ({ ...prev, [branchKey]: true }))

    try {
      const branches = await ApiClient.Chat.getMessageBranches({
        message_id: msg.id,
      })

      setMessageBranches(prev => ({ ...prev, [branchKey]: branches }))
    } catch (error) {
      message.error('Failed to load message branches')
    } finally {
      setLoadingBranches(prev => ({ ...prev, [branchKey]: false }))
    }
  }

  const handleSwitchBranch = async (messageId: string) => {
    try {
      await ApiClient.Chat.switchBranch({ message_id: messageId })

      // Reload conversation to show the new active branch
      if (conversation) {
        const conversationResponse = await ApiClient.Chat.getConversation({
          conversation_id: conversation.id,
        })
        setMessages(conversationResponse.messages)

        // Clear message branches cache to force reload of branch info
        setMessageBranches({})
        setLoadingBranches({})
      }

      message.success('Switched to selected branch')
    } catch (error) {
      message.error('Failed to switch branch')
    }
  }

  // Empty state (no conversation loaded)
  if (!conversation && !conversationId) {
    return (
      <ChatWelcome
        inputValue={inputValue}
        selectedAssistant={selectedAssistant}
        selectedModel={selectedModel}
        assistants={assistants}
        modelProviders={modelProviders}
        onInputChange={setInputValue}
        onAssistantChange={setSelectedAssistant}
        onModelChange={setSelectedModel}
        onSend={handleSend}
        onKeyPress={handleKeyPress}
      />
    )
  }

  return (
    <Flex className="flex-col h-dvh gap-3 relative">
      <div className={'absolute top-0 left-0 w-full z-10 backdrop-blur-2xl'}>
        <ChatHeader
          conversation={conversation}
          selectedAssistant={selectedAssistant}
          selectedModel={selectedModel}
          assistants={assistants}
          modelProviders={modelProviders}
        />
      </div>
      <Flex
        className={
          'max-w-4xl self-center w-full flex-1 h-full overflow-auto !pt-20 !mb-10'
        }
      >
        <ChatMessageList
          messages={messages}
          isLoading={isLoading}
          isStreaming={isStreaming}
          editingMessage={editingMessage}
          editValue={editValue}
          messageBranches={messageBranches}
          loadingBranches={loadingBranches}
          onEditMessage={handleEditMessage}
          onSaveEdit={handleSaveEdit}
          onCancelEdit={handleCancelEdit}
          onEditValueChange={setEditValue}
          onLoadBranches={loadMessageBranches}
          onSwitchBranch={handleSwitchBranch}
        />
      </Flex>
      <div className={'absolute bottom-0 w-full pb-2 justify-items-center'}>
        <div className={'max-w-4xl w-full'}>
          <ChatInput
            value={inputValue}
            onChange={setInputValue}
            onSend={handleSend}
            onStop={handleStopGeneration}
            onKeyPress={handleKeyPress}
            disabled={isLoading || isStreaming}
            isLoading={isLoading}
            isStreaming={isStreaming}
          />
        </div>
      </div>
    </Flex>
  )
}
