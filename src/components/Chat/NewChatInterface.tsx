import { useEffect, useState } from 'react'
import { App, Col, Row, Select, Space, Typography } from 'antd'
import { useTranslation } from 'react-i18next'
import { useNavigate } from 'react-router-dom'
import { useShallow } from 'zustand/react/shallow'
import { RobotOutlined } from '@ant-design/icons'
import { ChatInput } from './ChatInput'
import { useChatStore } from '../../store/chat'
import { useAssistantsStore } from '../../store/assistants'
import { useProvidersStore } from '../../store/modelProviders'
import { useConversationsStore } from '../../store'
import { useAuthStore } from '../../store/auth'

const { Text } = Typography
const { Option } = Select

export function NewChatInterface() {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const navigate = useNavigate()

  // Auth store
  const { user } = useAuthStore()

  // Chat store
  const {
    createConversation,
    loadConversation,
    sendMessage,
    error: chatError,
    clearError: clearChatError,
  } = useChatStore(
    useShallow(state => ({
      createConversation: state.createConversation,
      loadConversation: state.loadConversation,
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
    modelsByProvider,
    loading: providersLoading,
    loadProviders,
  } = useProvidersStore(
    useShallow(state => ({
      providers: state.providers,
      modelsByProvider: state.modelsByProvider,
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
        const providerModels = modelsByProvider[firstProvider.id] || []
        if (firstProvider && providerModels.length > 0) {
          const enabledModels = providerModels.filter(m => m.enabled)
          if (enabledModels.length > 0) {
            setSelectedModel(`${firstProvider.id}:${enabledModels[0].id}`)
          }
        }
      }
    }
  }, [modelProviders, selectedModel, modelsByProvider])

  const createNewConversation = async () => {
    if (!selectedAssistant || !selectedModel) {
      message.error(t('chat.noAssistantSelected'))
      return null
    }

    const [, modelId] = selectedModel.split(':')

    try {
      const conversationId = await createConversation(
        selectedAssistant,
        modelId,
      )

      // Add to conversations store immediately
      addConversation({
        id: conversationId,
        title: 'New Conversation', // This will be updated by the backend
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

    const [, modelId] = selectedModel.split(':')

    try {
      // Send the first message
      await sendMessage(inputValue.trim(), selectedAssistant, modelId)
      await loadConversation(conversationId, false)
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
    <div className="flex flex-col h-full">
      {/* Header with model selection */}
      <div className="px-4 sm:px-6 py-4">
        <Row gutter={16} align="middle">
          <Col xs={24} sm={12} md={8}>
            <Select
              value={selectedAssistant}
              onChange={setSelectedAssistant}
              placeholder={t('chat.selectAssistant')}
              className="w-full"
              showSearch
              optionFilterProp="children"
            >
              {userAssistants.map(assistant => (
                <Option key={assistant.id} value={assistant.id}>
                  <Space>
                    <RobotOutlined />
                    {assistant.name}
                  </Space>
                </Option>
              ))}
            </Select>
          </Col>
          <Col xs={24} sm={12} md={8}>
            <Select
              value={selectedModel}
              onChange={setSelectedModel}
              placeholder={t('chat.selectModel')}
              className="w-full"
              showSearch
              optionFilterProp="children"
            >
              {enabledProviders.map(provider => {
                const providerModels = modelsByProvider[provider.id] || []
                const enabledModels = providerModels.filter(
                  model => model.enabled,
                )

                if (enabledModels.length === 0) return null

                return (
                  <Select.OptGroup key={provider.id} label={provider.name}>
                    {enabledModels.map(model => (
                      <Option
                        key={`${provider.id}:${model.id}`}
                        value={`${provider.id}:${model.id}`}
                      >
                        {model.alias}
                      </Option>
                    ))}
                  </Select.OptGroup>
                )
              })}
            </Select>
          </Col>
        </Row>
      </div>

      {/* Welcome message */}
      <div className="flex flex-col items-center justify-center flex-1 text-center p-8">
        <div className="mb-8">
          <div className="text-3xl font-light mb-4">
            {t('chat.placeholderWelcome')}
          </div>
        </div>

        <div className="w-full max-w-2xl">
          <ChatInput
            onSend={handleSend}
            placeholder={t('chat.placeholder')}
            disabled={!selectedAssistant || !selectedModel}
          />

          {(!selectedAssistant || !selectedModel) && (
            <div className="mt-4">
              <Text type="secondary" className="text-sm">
                {t('chat.noAssistantSelected')}
              </Text>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
