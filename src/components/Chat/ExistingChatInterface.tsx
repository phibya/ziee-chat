import { useEffect, useState } from 'react'
import { App, Flex } from 'antd'
import { useTranslation } from 'react-i18next'
import { useShallow } from 'zustand/react/shallow'
import { ChatHeader } from './ChatHeader'
import { ChatMessageList } from './ChatMessageList'
import { ChatInput } from './ChatInput'
import { useChatStore } from '../../store/chat'
import { useAssistantsStore } from '../../store/assistants'
import { useModelProvidersStore } from '../../store/modelProviders'
import { useConversationsStore } from '../../store'

interface ExistingChatInterfaceProps {
  conversationId: string
}

export function ExistingChatInterface({
  conversationId,
}: ExistingChatInterfaceProps) {
  const { t } = useTranslation()
  const { message } = App.useApp()

  // Chat store
  const {
    currentConversation,
    currentMessages,
    loading: chatLoading,
    sending,
    error: chatError,
    loadConversation,
    sendMessage,
    editMessage,
    loadMessageBranches,
    switchBranch,
    clearError: clearChatError,
  } = useChatStore(
    useShallow(state => ({
      currentConversation: state.currentConversation,
      currentMessages: state.currentMessages,
      loading: state.loading,
      sending: state.sending,
      error: state.error,
      loadConversation: state.loadConversation,
      sendMessage: state.sendMessage,
      editMessage: state.editMessage,
      loadMessageBranches: state.loadMessageBranches,
      switchBranch: state.switchBranch,
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
  const { updateConversation } = useConversationsStore()

  const [selectedAssistant, setSelectedAssistant] = useState<string | null>(
    null,
  )
  const [selectedModel, setSelectedModel] = useState<string | null>(null)
  const [editingMessage, setEditingMessage] = useState<string | null>(null)
  const [editValue, setEditValue] = useState('')
  const [messageBranches, setMessageBranches] = useState<{
    [key: string]: any[]
  }>({})

  useEffect(() => {
    initializeData()
    return () => {}
  }, [])

  useEffect(() => {
    if (conversationId) {
      loadConversation(conversationId)
    }
  }, [conversationId])

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
      message.error(error?.message || t('common.failedToLoadData'))
    }
  }

  // Update selected assistant and model when conversation loads
  useEffect(() => {
    if (currentConversation) {
      if (currentConversation.assistant_id) {
        setSelectedAssistant(currentConversation.assistant_id)
      }
      if (currentConversation.model_id) {
        // Find the provider that contains this model
        const provider = modelProviders.find(p =>
          p.models?.some(m => m.id === currentConversation.model_id),
        )
        if (provider) {
          setSelectedModel(`${provider.id}:${currentConversation.model_id}`)
        }
      }
    }
  }, [currentConversation, modelProviders])

  const handleSendMessage = async (inputValue: string) => {
    if (!currentConversation || !selectedAssistant || !selectedModel) return

    const [, modelId] = selectedModel.split(':')

    try {
      await sendMessage(inputValue.trim(), selectedAssistant, modelId)

      // Update conversation in store with new title and last message if it changed
      if (currentConversation && currentMessages.length > 0) {
        const lastMessage = currentMessages[currentMessages.length - 1]
        await updateConversation(currentConversation.id, {
          title: currentConversation.title,
          updated_at: new Date().toISOString(),
          last_message: lastMessage.content.substring(0, 100),
          message_count: currentMessages.length,
        })
      }
    } catch (error) {
      console.error('Chat error:', error)
    }
  }

  const handleEditMessage = (messageId: string, content: string) => {
    setEditingMessage(messageId)
    setEditValue(content)
  }

  const handleSaveEdit = async () => {
    if (
      !editingMessage ||
      !editValue.trim() ||
      !selectedAssistant ||
      !selectedModel
    )
      return

    // No longer needed after API simplification
    // const [providerId, modelId] = selectedModel.split(':')

    try {
      const originalMessage = currentMessages.find(m => m.id === editingMessage)
      const contentChanged =
        originalMessage && originalMessage.content.trim() !== editValue.trim()

      await editMessage(editingMessage, editValue.trim())

      setEditingMessage(null)
      setEditValue('')

      if (contentChanged) {
        message.success(t('chat.messageUpdatedAndSent'))
      } else {
        message.success(t('chat.messageUpdated'))
      }
    } catch (error) {
      // Error is already handled by the store
      console.error('Failed to update message:', error)
    }
  }

  const handleCancelEdit = () => {
    setEditingMessage(null)
    setEditValue('')
  }

  const handleLoadBranches = async (msg: any) => {
    if (!currentConversation) return

    const branchKey = `${msg.id}`

    try {
      const branches = await loadMessageBranches(msg.id)
      setMessageBranches(prev => ({ ...prev, [branchKey]: branches }))
    } catch (error) {
      // Error is already handled by the store
      console.error('Failed to load message branches:', error)
    }
  }

  const handleSwitchBranch = async (branchId: string) => {
    if (!currentConversation) return
    try {
      await switchBranch(currentConversation.id, branchId)

      // Clear message branches cache to force reload of branch info
      setMessageBranches({})

      message.success(t('chat.switchedToBranch'))
    } catch (error) {
      // Error is already handled by the store
      console.error('Failed to switch branch:', error)
    }
  }

  if (chatLoading || assistantsLoading || providersLoading) {
    return <div>Loading...</div>
  }

  if (!currentConversation) {
    return <div>Conversation not found</div>
  }

  // Filter to only show user's own assistants in chat (not admin templates)
  const userAssistants = assistants.filter(a => !a.is_template)

  // Filter to only show enabled providers
  const enabledProviders = modelProviders.filter(p => p.enabled)

  return (
    <Flex className="flex-col h-dvh gap-3 relative">
      <div className={'absolute top-0 left-0 w-full z-10 backdrop-blur-2xl'}>
        <ChatHeader
          conversation={currentConversation}
          selectedAssistant={selectedAssistant}
          selectedModel={selectedModel}
          assistants={userAssistants}
          modelProviders={enabledProviders}
        />
      </div>
      <Flex
        className={
          'max-w-4xl self-center w-full flex-1 h-full overflow-auto !pt-20 !mb-10'
        }
      >
        <ChatMessageList
          messages={currentMessages}
          isLoading={sending}
          isStreaming={sending}
          editingMessage={editingMessage}
          editValue={editValue}
          messageBranches={messageBranches}
          loadingBranches={{}}
          onEditMessage={handleEditMessage}
          onSaveEdit={handleSaveEdit}
          onCancelEdit={handleCancelEdit}
          onEditValueChange={setEditValue}
          onLoadBranches={handleLoadBranches}
          onSwitchBranch={handleSwitchBranch}
        />
      </Flex>
      <div className={'absolute bottom-0 w-full pb-2 justify-items-center'}>
        <div className={'max-w-4xl w-full'}>
          <ChatInput onSend={handleSendMessage} disabled={sending} />
        </div>
      </div>
    </Flex>
  )
}
