import { useEffect, useRef, useState } from 'react'
import { Button, Input, Space, Spin, Typography, Select, Row, Col } from 'antd'
import {
  LoadingOutlined,
  MessageOutlined,
  RobotOutlined,
  SendOutlined,
  StopOutlined,
  SaveOutlined,
  CloseOutlined,
  LeftOutlined,
  RightOutlined,
} from '@ant-design/icons'
import { useNavigate } from 'react-router-dom'
import { ApiClient } from '../../api/client'
import { useTheme } from '../../hooks/useTheme'
import {
  Conversation,
  Message,
  CreateConversationRequest,
} from '../../types/api/chat'
import { Assistant } from '../../types/api/assistant'
import { ModelProvider } from '../../types/api/modelProvider'
import { User } from '../../types/api/user'
import { App } from 'antd'
import ReactMarkdown from 'react-markdown'

const { TextArea } = Input
const { Text } = Typography
const { Option } = Select

interface ChatInterfaceProps {
  conversationId: string | null
}

export function ChatInterface({ conversationId }: ChatInterfaceProps) {
  const appTheme = useTheme()
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
  const messagesEndRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    initializeData()
  }, [])

  useEffect(() => {
    if (conversationId) {
      loadConversation(conversationId)
    }
  }, [conversationId])

  useEffect(() => {
    scrollToBottom()
  }, [messages])

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

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }

  const createNewConversation = async () => {
    if (!selectedAssistant || !selectedModel) {
      message.error('Please select an assistant and model')
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

    try {
      // Send the message to the backend
      await ApiClient.Chat.sendMessage({
        conversation_id: currentConversation.id,
        content: userInput,
        model_provider_id: providerId,
        model_id: modelId,
      })

      // Reload the conversation to get the updated messages
      const conversationResponse = await ApiClient.Chat.getConversation({
        conversation_id: currentConversation.id,
      })

      setMessages(conversationResponse.messages)

      // Reset loading states after successful completion
      setIsLoading(false)
      setIsStreaming(false)
    } catch (error) {
      message.error('Failed to send message')
      setIsLoading(false)
      setIsStreaming(false)
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

    const branchKey = `${msg.created_at}`

    setLoadingBranches(prev => ({ ...prev, [branchKey]: true }))

    try {
      const timestamp = new Date(msg.created_at).toISOString()
      const branches = await ApiClient.Chat.getMessageBranches({
        conversation_id: conversation.id,
        timestamp: timestamp,
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

  const getBranchInfo = (msg: Message) => {
    const branchKey = `${msg.created_at}`
    const branches = messageBranches[branchKey] || []
    const currentIndex = branches.findIndex(b => b.is_active_branch)

    return {
      branches,
      currentIndex,
      hasBranches: branches.length > 1,
      isLoading: loadingBranches[branchKey] || false,
    }
  }

  const renderMessage = (msg: Message) => {
    const isUser = msg.role === 'user'
    const isEditing = editingMessage === msg.id

    return (
      <div key={msg.id} className="mb-6 group">
        <div
          className="rounded-lg px-4 py-3"
          style={{
            backgroundColor: isUser ? 'rgba(255,255,255,0.05)' : 'transparent',
          }}
        >
          {/* Message header with avatar */}
          <div className="flex items-center gap-3 mb-2">
            <div className="flex items-center gap-2">
              <div
                className="w-8 h-8 rounded-full flex items-center justify-center text-sm font-medium"
                style={{
                  backgroundColor: isUser ? '#666' : '#f97316',
                  color: 'white',
                }}
              >
                {isUser ? 'P' : <RobotOutlined />}
              </div>
            </div>
          </div>

          {/* Message content */}
          <div className="ml-11">
            {isEditing ? (
              <div className="space-y-3">
                <TextArea
                  value={editValue}
                  onChange={e => setEditValue(e.target.value)}
                  autoSize={{ minRows: 2, maxRows: 8 }}
                  className="w-full"
                  style={{
                    backgroundColor: 'rgba(255,255,255,0.05)',
                    border: '1px solid rgba(255,255,255,0.1)',
                    color: 'inherit',
                  }}
                />
                <div className="flex gap-2">
                  <Button
                    size="small"
                    type="primary"
                    icon={<SaveOutlined />}
                    onClick={handleSaveEdit}
                  >
                    Save
                  </Button>
                  <Button
                    size="small"
                    icon={<CloseOutlined />}
                    onClick={handleCancelEdit}
                  >
                    Cancel
                  </Button>
                </div>
              </div>
            ) : (
              <div
                className="text-base leading-relaxed"
                style={{
                  color: 'rgba(255,255,255,0.9)',
                  whiteSpace: isUser ? 'pre-wrap' : 'normal',
                }}
              >
                {isUser ? (
                  msg.content
                ) : (
                  <ReactMarkdown>{msg.content}</ReactMarkdown>
                )}
              </div>
            )}
          </div>

          {/* Tools/Actions at the bottom for user messages */}
          {isUser &&
            !isEditing &&
            (() => {
              const branchInfo = getBranchInfo(msg)
              return (
                <div className="ml-11 mt-2 flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                  <Button
                    size="small"
                    type="text"
                    onClick={() => handleEditMessage(msg.id, msg.content)}
                    className="text-xs px-2 h-6"
                    style={{
                      backgroundColor: 'rgba(255,255,255,0.1)',
                      border: '1px solid rgba(255,255,255,0.2)',
                      color: 'rgba(255,255,255,0.8)',
                    }}
                  >
                    Edit
                  </Button>

                  {!branchInfo.hasBranches && !branchInfo.isLoading && (
                    <Button
                      size="small"
                      type="text"
                      onClick={() => loadMessageBranches(msg)}
                      className="w-6 h-6 min-w-0 p-0"
                      style={{
                        backgroundColor: 'rgba(255,255,255,0.1)',
                        border: '1px solid rgba(255,255,255,0.2)',
                        color: 'rgba(255,255,255,0.8)',
                      }}
                    >
                      <LeftOutlined />
                    </Button>
                  )}

                  {branchInfo.isLoading && <Spin size="small" />}

                  {branchInfo.hasBranches && (
                    <>
                      <Button
                        size="small"
                        type="text"
                        icon={<LeftOutlined />}
                        disabled={branchInfo.currentIndex <= 0}
                        onClick={() => {
                          const prevBranch =
                            branchInfo.branches[branchInfo.currentIndex - 1]
                          if (prevBranch) handleSwitchBranch(prevBranch.id)
                        }}
                        className="w-6 h-6 min-w-0 p-0"
                        style={{
                          backgroundColor: 'rgba(255,255,255,0.1)',
                          border: '1px solid rgba(255,255,255,0.2)',
                          color: 'rgba(255,255,255,0.8)',
                        }}
                      />
                      <div
                        className="text-xs px-2 h-6 flex items-center"
                        style={{
                          backgroundColor: 'rgba(255,255,255,0.1)',
                          border: '1px solid rgba(255,255,255,0.2)',
                          color: 'rgba(255,255,255,0.8)',
                          borderRadius: '6px',
                        }}
                      >
                        {branchInfo.currentIndex + 1} /{' '}
                        {branchInfo.branches.length}
                      </div>
                      <Button
                        size="small"
                        type="text"
                        icon={<RightOutlined />}
                        disabled={
                          branchInfo.currentIndex >=
                          branchInfo.branches.length - 1
                        }
                        onClick={() => {
                          const nextBranch =
                            branchInfo.branches[branchInfo.currentIndex + 1]
                          if (nextBranch) handleSwitchBranch(nextBranch.id)
                        }}
                        className="w-6 h-6 min-w-0 p-0"
                        style={{
                          backgroundColor: 'rgba(255,255,255,0.1)',
                          border: '1px solid rgba(255,255,255,0.2)',
                          color: 'rgba(255,255,255,0.8)',
                        }}
                      />
                    </>
                  )}
                </div>
              )
            })()}
        </div>
      </div>
    )
  }

  // Empty state (no conversation loaded)
  if (!conversation && !conversationId) {
    return (
      <div
        className="flex flex-col h-full"
        style={{ backgroundColor: appTheme.chatBackground }}
      >
        {/* Header with model selection */}
        <div
          className="px-4 sm:px-6 py-4 border-b"
          style={{
            backgroundColor: appTheme.surface,
            borderColor: appTheme.borderSecondary,
          }}
        >
          <Row gutter={16} align="middle">
            <Col xs={24} sm={12} md={8}>
              <div className="mb-2">
                <Text strong>My Assistant</Text>
              </div>
              <Select
                value={selectedAssistant}
                onChange={setSelectedAssistant}
                placeholder="Select your assistant"
                className="w-full"
                showSearch
                optionFilterProp="children"
              >
                {assistants.map(assistant => (
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
              <div className="mb-2">
                <Text strong>Model</Text>
              </div>
              <Select
                value={selectedModel}
                onChange={setSelectedModel}
                placeholder="Select a model"
                className="w-full"
                showSearch
                optionFilterProp="children"
              >
                {modelProviders.map(provider => (
                  <Select.OptGroup key={provider.id} label={provider.name}>
                    {provider.models.map(model => (
                      <Option
                        key={`${provider.id}:${model.id}`}
                        value={`${provider.id}:${model.id}`}
                      >
                        {model.alias}
                      </Option>
                    ))}
                  </Select.OptGroup>
                ))}
              </Select>
            </Col>
          </Row>
        </div>

        {/* Welcome message */}
        <div className="flex flex-col items-center justify-center flex-1 text-center p-8">
          <div className="mb-8">
            <div
              className="text-3xl font-light mb-4"
              style={{ color: 'rgba(255,255,255,0.9)' }}
            >
              What can I help with?
            </div>
          </div>

          <div className="w-full max-w-2xl">
            <div className="flex items-end gap-3">
              <div className="flex-1">
                <TextArea
                  value={inputValue}
                  onChange={e => setInputValue(e.target.value)}
                  onKeyPress={handleKeyPress}
                  placeholder="Message Claude..."
                  autoSize={{ minRows: 1, maxRows: 6 }}
                  disabled={!selectedAssistant || !selectedModel}
                  style={{
                    backgroundColor: 'rgba(255,255,255,0.05)',
                    border: '1px solid rgba(255,255,255,0.1)',
                    borderRadius: '12px',
                    color: 'inherit',
                    padding: '12px 16px',
                    fontSize: '16px',
                  }}
                  className="resize-none"
                />
              </div>
              <Button
                type="primary"
                icon={<SendOutlined />}
                onClick={handleSend}
                disabled={
                  !inputValue.trim() || !selectedAssistant || !selectedModel
                }
                style={{
                  backgroundColor:
                    !inputValue.trim() || !selectedAssistant || !selectedModel
                      ? 'rgba(255,255,255,0.1)'
                      : '#f97316',
                  borderColor:
                    !inputValue.trim() || !selectedAssistant || !selectedModel
                      ? 'rgba(255,255,255,0.2)'
                      : '#f97316',
                  borderRadius: '8px',
                  height: '40px',
                }}
              >
                Send
              </Button>
            </div>

            {(!selectedAssistant || !selectedModel) && (
              <div className="mt-4">
                <Text
                  style={{ color: 'rgba(255,255,255,0.6)', fontSize: '14px' }}
                >
                  Please select an assistant and model to start chatting
                </Text>
              </div>
            )}
          </div>
        </div>
      </div>
    )
  }

  return (
    <div
      className="flex flex-col h-full"
      style={{ backgroundColor: appTheme.chatBackground }}
    >
      {/* Header with conversation title and controls */}
      <div className="px-4 sm:px-6 py-4">
        <div className="max-w-4xl mx-auto">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <Text
                strong
                className="text-lg"
                style={{ color: 'rgba(255,255,255,0.9)' }}
              >
                {conversation?.title || 'Claude'}
              </Text>
            </div>

            <div
              className="flex items-center gap-2 text-sm"
              style={{ color: 'rgba(255,255,255,0.6)' }}
            >
              <span>
                {selectedAssistant &&
                  assistants.find(a => a.id === selectedAssistant)?.name}
              </span>
              <span>•</span>
              <span>
                {selectedModel &&
                  (() => {
                    const [providerId, modelId] = selectedModel.split(':')
                    const provider = modelProviders.find(
                      p => p.id === providerId,
                    )
                    const model = provider?.models.find(m => m.id === modelId)
                    return model?.alias || modelId
                  })()}
              </span>
            </div>
          </div>
        </div>
      </div>

      {/* Messages */}
      <div className="flex-1 overflow-auto">
        <div className="max-w-4xl mx-auto px-4 sm:px-6 py-6">
          {messages.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-full text-center py-20">
              <MessageOutlined
                className="text-5xl mb-4"
                style={{ color: 'rgba(255,255,255,0.3)' }}
              />
              <Text
                className="text-lg"
                style={{ color: 'rgba(255,255,255,0.6)' }}
              >
                Start your conversation
              </Text>
            </div>
          ) : (
            <>
              {messages.map(renderMessage)}
              {(isLoading || isStreaming) && (
                <div className="mb-6">
                  <div className="flex items-center gap-3 mb-2">
                    <div
                      className="w-8 h-8 rounded-full flex items-center justify-center text-sm font-medium"
                      style={{
                        backgroundColor: '#f97316',
                        color: 'white',
                      }}
                    >
                      <RobotOutlined />
                    </div>
                  </div>
                  <div className="ml-11">
                    <div
                      className="flex items-center gap-2 text-base"
                      style={{ color: 'rgba(255,255,255,0.7)' }}
                    >
                      <Spin
                        indicator={
                          <LoadingOutlined style={{ fontSize: 16 }} spin />
                        }
                      />
                      <span>
                        {isStreaming ? 'Generating...' : 'Thinking...'}
                      </span>
                    </div>
                  </div>
                </div>
              )}
              <div ref={messagesEndRef} />
            </>
          )}
        </div>
      </div>

      {/* Input */}
      <div className="px-4 sm:px-6 py-4">
        <div className="max-w-4xl mx-auto">
          <div className="flex items-end gap-3">
            <div className="flex-1">
              <TextArea
                value={inputValue}
                onChange={e => setInputValue(e.target.value)}
                onKeyPress={handleKeyPress}
                placeholder="Message Claude..."
                autoSize={{ minRows: 1, maxRows: 6 }}
                disabled={isLoading || isStreaming}
                style={{
                  backgroundColor: 'rgba(255,255,255,0.05)',
                  border: '1px solid rgba(255,255,255,0.1)',
                  borderRadius: '12px',
                  color: 'inherit',
                  padding: '12px 16px',
                }}
                className="resize-none"
              />
            </div>
            <div className="flex gap-2">
              {(isLoading || isStreaming) && (
                <Button
                  type="text"
                  icon={<StopOutlined />}
                  onClick={handleStopGeneration}
                  style={{
                    backgroundColor: 'rgba(255,255,255,0.1)',
                    border: '1px solid rgba(255,255,255,0.2)',
                    color: 'rgba(255,255,255,0.8)',
                    borderRadius: '8px',
                  }}
                >
                  Stop
                </Button>
              )}
              <Button
                type="primary"
                icon={<SendOutlined />}
                onClick={handleSend}
                disabled={!inputValue.trim() || isLoading || isStreaming}
                style={{
                  backgroundColor:
                    !inputValue.trim() || isLoading || isStreaming
                      ? 'rgba(255,255,255,0.1)'
                      : '#f97316',
                  borderColor:
                    !inputValue.trim() || isLoading || isStreaming
                      ? 'rgba(255,255,255,0.2)'
                      : '#f97316',
                  borderRadius: '8px',
                }}
              >
                Send
              </Button>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
