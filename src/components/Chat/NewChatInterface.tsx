import { useEffect, useState } from 'react'
import { App, Col, Row, Select, Flex, Typography } from 'antd'
import { useTranslation } from 'react-i18next'
import { useNavigate } from 'react-router-dom'
import { RobotOutlined } from '@ant-design/icons'
import { ChatInput } from './ChatInput'
import {
  Stores,
  createNewConversation,
  loadConversationById,
  sendChatMessage,
  clearChatError,
  loadUserAssistants,
  loadAllModelProviders,
  addNewConversationToList,
} from '../../store'

const { Text } = Typography
const { Option } = Select

export function NewChatInterface() {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const navigate = useNavigate()

  // Auth store
  const { user } = Stores.Auth

  // Chat store
  const { error: chatError } = Stores.Chat

  // Assistants store
  const { assistants, loading: assistantsLoading } = Stores.Assistants

  // Model providers store
  const {
    providers: providers,
    modelsByProvider,
    loading: providersLoading,
  } = Stores.Providers

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
  }, [chatError]) // Removed message and clearChatError from dependencies to prevent infinite rerenders

  const initializeData = async () => {
    try {
      await Promise.all([loadUserAssistants(), loadAllModelProviders()])
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
    if (providers.length > 0 && !selectedModel) {
      // Filter to only show enabled providers
      const enabledProviders = providers.filter(p => p.enabled)
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
  }, [providers, selectedModel, modelsByProvider])

  const handleCreateNewConversation = async (): Promise<string | null> => {
    if (!selectedAssistant || !selectedModel) {
      message.error(t('chat.noAssistantSelected'))
      return null
    }

    const [, modelId] = selectedModel.split(':')

    try {
      const conversationId = await createNewConversation(
        selectedAssistant,
        modelId,
      )

      // Add to conversations store immediately
      addNewConversationToList({
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
    const conversationId = await handleCreateNewConversation()
    if (!conversationId) return

    const [, modelId] = selectedModel.split(':')

    try {
      // Send the first message
      await sendChatMessage(inputValue.trim(), selectedAssistant, modelId)
      await loadConversationById(conversationId, false)
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
  const enabledProviders = providers.filter(p => p.enabled)

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
                  <Flex className="gap-2">
                    <RobotOutlined />
                    {assistant.name}
                  </Flex>
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
