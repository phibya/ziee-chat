import { useEffect, useState } from 'react'
import { App, Flex } from 'antd'
import { ApiClient } from '../../api/client'
import { Conversation, Message } from '../../types/api/chat'
import { Assistant } from '../../types/api/assistant'
import { ModelProvider } from '../../types/api/modelProvider'
import { User } from '../../types/api/user'
import { ChatHeader } from './ChatHeader'
import { ChatMessageList } from './ChatMessageList'
import { ChatInput } from './ChatInput'
import { useConversationsStore } from '../../store'

interface ExistingChatInterfaceProps {
  conversationId: string
}

export function ExistingChatInterface({
  conversationId,
}: ExistingChatInterfaceProps) {
  const { message } = App.useApp()
  const { updateConversation } = useConversationsStore()

  const [conversation, setConversation] = useState<Conversation | null>(null)
  const [messages, setMessages] = useState<Message[]>([])
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
      const availableProviders = providersResponse.providers.filter(
        p => p.enabled,
      )
      setModelProviders(availableProviders)
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

  const handleSendMessage = async (inputValue: string) => {
    if (!conversation || !selectedModel) return

    const [providerId, modelId] = selectedModel.split(':')

    try {
      // Send message using the API
      await ApiClient.Chat.sendMessage({
        conversation_id: conversation.id,
        content: inputValue.trim(),
        model_provider_id: providerId,
        model_id: modelId,
      })

      // Reload the conversation to get the actual messages
      const conversationResponse = await ApiClient.Chat.getConversation({
        conversation_id: conversation.id,
      })

      setMessages(conversationResponse.messages)

      // Update conversation in store with new title and last message if it changed
      if (conversationResponse.conversation.title !== conversation.title) {
        updateConversation(conversation.id, {
          title: conversationResponse.conversation.title,
          updated_at: conversationResponse.conversation.updated_at,
          last_message:
            conversationResponse.messages.length > 0
              ? conversationResponse.messages[
                  conversationResponse.messages.length - 1
                ].content.substring(0, 100)
              : undefined,
          message_count: conversationResponse.messages.length,
        })
      }
    } catch (error) {
      console.error('Chat error:', error)
      message.error(
        'Failed to send message: ' +
          (error instanceof Error ? error.message : 'Unknown error'),
      )
    }
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

  if (!conversation) {
    return <div>Loading...</div>
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
          isLoading={false}
          isStreaming={false}
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
          <ChatInput onSend={handleSendMessage} />
        </div>
      </div>
    </Flex>
  )
}
