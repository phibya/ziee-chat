import { memo, useState, useEffect } from 'react'
import { Button, Flex, Input, Select } from 'antd'
import { useTranslation } from 'react-i18next'
import { useNavigate } from 'react-router-dom'
import { SendOutlined, StopOutlined } from '@ant-design/icons'
import {
  Stores,
  sendChatMessage,
  stopMessageStreaming,
  createNewConversation,
  loadConversationById,
  addNewConversationToList,
} from '../../store'

const { TextArea } = Input

interface ChatInputProps {
  projectId?: string
  onNewConversationCreated?: (id: string) => void
}

export const ChatInput = memo(function ChatInput({
  projectId,
  onNewConversationCreated,
}: ChatInputProps) {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const [inputValue, setInputValue] = useState('')
  const [selectedAssistant, setSelectedAssistant] = useState<string>()
  const [selectedModel, setSelectedModel] = useState<string>()

  const { currentConversation, sending, isStreaming } = Stores.Chat
  const { assistants } = Stores.Assistants
  const { providers, modelsByProvider } = Stores.Providers
  const { user } = Stores.Auth

  // Get available assistants (filter out templates)
  const getAvailableAssistants = () => {
    return assistants.filter(a => !a.is_template)
  }

  // Get available models for current providers
  const getAvailableModels = () => {
    const enabledProviders = providers.filter(p => p.enabled)
    const models: Array<{ label: string; value: string; provider: string }> = []

    enabledProviders.forEach(provider => {
      const providerModels = modelsByProvider[provider.id] || []
      const enabledModels = providerModels.filter(m => m.enabled)

      enabledModels.forEach(model => {
        models.push({
          label: `${provider.name} - ${model.alias || model.id}`,
          value: `${provider.id}:${model.id}`,
          provider: provider.name,
        })
      })
    })

    return models
  }

  // Initialize default selections
  useEffect(() => {
    if (!selectedAssistant) {
      const availableAssistants = getAvailableAssistants()
      if (availableAssistants.length > 0) {
        setSelectedAssistant(availableAssistants[0].id)
      }
    }
  }, [assistants, selectedAssistant])

  useEffect(() => {
    if (!selectedModel) {
      const availableModels = getAvailableModels()
      if (availableModels.length > 0) {
        setSelectedModel(availableModels[0].value)
      }
    }
  }, [providers, modelsByProvider, selectedModel])

  // For existing conversations, sync selections with conversation data
  useEffect(() => {
    if (currentConversation) {
      setSelectedAssistant(currentConversation.assistant_id)
      // Find the provider for this model
      const availableModels = getAvailableModels()
      const matchingModel = availableModels.find(model =>
        model.value.endsWith(`:${currentConversation.model_id}`),
      )
      if (matchingModel) {
        setSelectedModel(matchingModel.value)
      }
    }
  }, [currentConversation])

  const handleCreateNewConversation = async (): Promise<string | null> => {
    if (!selectedAssistant || !selectedModel) return null

    const [, modelId] = selectedModel.split(':')

    try {
      const conversationId = await createNewConversation(
        selectedAssistant,
        modelId,
        projectId,
      )

      // Add to conversations store immediately
      addNewConversationToList({
        id: conversationId,
        title: 'New Conversation',
        user_id: user?.id || '',
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
        last_message: undefined,
        message_count: 0,
      })

      // Call the callback if provided, otherwise navigate
      if (onNewConversationCreated) {
        onNewConversationCreated(conversationId)
      } else {
        navigate(`/conversation/${conversationId}`)
      }

      return conversationId
    } catch (error) {
      console.error('Failed to create conversation:', error)
      return null
    }
  }

  const handleSend = async () => {
    const messageToSend = inputValue.trim()
    if (!messageToSend || !selectedAssistant || !selectedModel) return

    setInputValue('') // Clear input immediately

    try {
      if (currentConversation) {
        // Existing conversation: send message directly
        await sendChatMessage(
          messageToSend,
          selectedAssistant,
          selectedModel.split(':')[1],
        )
      } else {
        // New conversation: create conversation then send message
        const conversationId = await handleCreateNewConversation()
        if (conversationId) {
          await sendChatMessage(
            messageToSend,
            selectedAssistant,
            selectedModel.split(':')[1],
          )
          await loadConversationById(conversationId, false)
        }
      }
    } catch (error) {
      console.error('Failed to send message:', error)
      // Restore the message if sending failed
      setInputValue(messageToSend)
    }
  }

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      handleSend()
    }
  }

  const availableAssistants = getAvailableAssistants()
  const availableModels = getAvailableModels()
  const isDisabled = sending
  const showStop = sending || isStreaming

  return (
    <Flex vertical className="w-full gap-2">
      <Flex gap="small">
        <Select
          value={selectedAssistant}
          onChange={setSelectedAssistant}
          placeholder="Select assistant"
          style={{ width: 200 }}
          disabled={isDisabled}
          options={availableAssistants.map(assistant => ({
            label: assistant.name,
            value: assistant.id,
          }))}
        />
        <Select
          value={selectedModel}
          onChange={setSelectedModel}
          placeholder="Select model"
          style={{ width: 250 }}
          disabled={isDisabled}
          options={availableModels.map(model => ({
            label: model.label,
            value: model.value,
          }))}
        />
      </Flex>
      <Flex className="flex items-end gap-1 w-full">
        <div className="flex-1">
          <TextArea
            value={inputValue}
            onChange={e => setInputValue(e.target.value)}
            onKeyPress={handleKeyPress}
            placeholder={t('chat.messageAI')}
            autoSize={{ minRows: 1, maxRows: 6 }}
            disabled={isDisabled}
            className="resize-none"
          />
        </div>
        <div className="flex gap-2">
          {showStop && (
            <Button
              type="text"
              icon={<StopOutlined />}
              onClick={stopMessageStreaming}
            >
              {t('chat.stop')}
            </Button>
          )}
          <Button
            type="primary"
            icon={<SendOutlined />}
            onClick={handleSend}
            disabled={
              !inputValue.trim() ||
              isDisabled ||
              !selectedAssistant ||
              !selectedModel
            }
            loading={sending}
          >
            {t('chat.send')}
          </Button>
        </div>
      </Flex>
    </Flex>
  )
})
