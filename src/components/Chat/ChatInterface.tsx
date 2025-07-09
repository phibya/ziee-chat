import { useEffect, useRef, useState } from 'react'
import {
  Avatar,
  Button,
  Card,
  Input,
  Space,
  Spin,
  Typography,
  Select,
  Row,
  Col,
  Tooltip,
  Divider,
} from 'antd'
import {
  LoadingOutlined,
  MessageOutlined,
  RobotOutlined,
  SendOutlined,
  UserOutlined,
  StopOutlined,
  EditOutlined,
  SaveOutlined,
  CloseOutlined,
  LeftOutlined,
  RightOutlined,
} from '@ant-design/icons'
import { useSearchParams } from 'react-router-dom'
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
  threadId: string | null
}

export function ChatInterface({ threadId: _ }: ChatInterfaceProps) {
  const appTheme = useTheme()
  const { message } = App.useApp()
  const [searchParams] = useSearchParams()
  const [inputValue, setInputValue] = useState('')
  const [isLoading, setIsLoading] = useState(false)
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
  const messagesEndRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    initializeData()
  }, [])

  useEffect(() => {
    const conversationId = searchParams.get('conversation')
    if (conversationId) {
      loadConversation(conversationId)
    }
  }, [searchParams])

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

      // Filter model providers based on user's group assignments
      const userGroupIds = userResponse.groups.map(g => g.id)
      const availableProviders = providersResponse.providers.filter(p => {
        // If user is in any group, check if the provider is assigned to any of their groups
        if (userGroupIds.length > 0) {
          return (
            p.enabled &&
            userResponse.groups.some(
              group =>
                group.model_provider_ids &&
                group.model_provider_ids.includes(p.id),
            )
          )
        }
        // If user is not in any groups, they can access all enabled providers (fallback)
        return p.enabled
      })
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

      setConversation(conversationResponse)
      setMessages(conversationResponse.messages)

      if (conversationResponse.assistant_id) {
        setSelectedAssistant(conversationResponse.assistant_id)
      }
      if (conversationResponse.model_provider_id) {
        setSelectedModel(
          `${conversationResponse.model_provider_id}:${conversationResponse.model_id}`,
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
      return newConversation
    } catch (error) {
      message.error('Failed to create conversation')
      return null
    }
  }

  const handleSend = async () => {
    if (!inputValue.trim()) return

    let currentConversation = conversation
    if (!currentConversation) {
      currentConversation = await createNewConversation()
      if (!currentConversation) return
    }

    try {
      const userMessage = await ApiClient.Chat.sendMessage({
        conversation_id: currentConversation.id,
        content: inputValue.trim(),
      })

      setMessages(prev => [...prev, userMessage])
      setInputValue('')
      setIsLoading(true)

      // Note: In a real implementation, this would stream the response
      // For now, we'll simulate an AI response
      setTimeout(async () => {
        try {
          const aiMessage = await ApiClient.Chat.sendMessage({
            conversation_id: currentConversation.id,
            content: `I received your message: "${inputValue.trim()}". This is a simulated response from the AI assistant. In a real implementation, this would be connected to your LLM backend.`,
          })

          setMessages(prev => [...prev, aiMessage])
        } catch (error) {
          message.error('Failed to get AI response')
        } finally {
          setIsLoading(false)
        }
      }, 1000)
    } catch (error) {
      message.error('Failed to send message')
      setIsLoading(false)
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
    message.info('Generation stopped')
  }

  const handleEditMessage = (messageId: string, content: string) => {
    setEditingMessage(messageId)
    setEditValue(content)
  }

  const handleSaveEdit = async () => {
    if (!editingMessage || !editValue.trim()) return

    try {
      await ApiClient.Chat.editMessage({
        message_id: editingMessage,
        content: editValue.trim(),
      })

      // Reload conversation to show the updated branch
      if (conversation) {
        const conversationResponse = await ApiClient.Chat.getConversation({
          conversation_id: conversation.id,
        })
        setMessages(conversationResponse.messages)
      }

      setEditingMessage(null)
      setEditValue('')
      message.success('Message updated successfully')
    } catch (error) {
      message.error('Failed to update message')
    }
  }

  const handleCancelEdit = () => {
    setEditingMessage(null)
    setEditValue('')
  }

  const renderMessage = (msg: Message) => {
    const isUser = msg.role === 'user'
    const isEditing = editingMessage === msg.id

    return (
      <div
        key={msg.id}
        className={`flex ${isUser ? 'justify-end' : 'justify-start'} mb-4 gap-3 group`}
      >
        {!isUser && (
          <Avatar
            size="small"
            icon={<RobotOutlined />}
            className="flex-shrink-0"
            style={{ backgroundColor: appTheme.success }}
          />
        )}

        <div className="max-w-[85%] sm:max-w-[70%] relative">
          {isEditing ? (
            <Card
              size="small"
              className="border-none rounded-xl"
              style={{
                backgroundColor: appTheme.chatMessageUser,
                color: appTheme.chatMessageUserText,
              }}
              bodyStyle={{ padding: '8px 12px' }}
            >
              <TextArea
                value={editValue}
                onChange={e => setEditValue(e.target.value)}
                autoSize={{ minRows: 2, maxRows: 8 }}
                className="mb-2"
              />
              <Space>
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
              </Space>
            </Card>
          ) : (
            <Card
              size="small"
              className="border-none rounded-xl"
              style={{
                backgroundColor: isUser
                  ? appTheme.chatMessageUser
                  : appTheme.chatMessageAssistant,
                color: isUser
                  ? appTheme.chatMessageUserText
                  : appTheme.chatMessageAssistantText,
              }}
              bodyStyle={{
                padding: '8px 12px',
                wordBreak: 'break-word',
              }}
            >
              {isUser ? (
                <div style={{ whiteSpace: 'pre-wrap' }}>{msg.content}</div>
              ) : (
                <ReactMarkdown>{msg.content}</ReactMarkdown>
              )}
            </Card>
          )}

          {/* Edit button for user messages */}
          {isUser && !isEditing && (
            <div className="absolute -right-8 top-2 opacity-0 group-hover:opacity-100 transition-opacity">
              <Tooltip title="Edit message">
                <Button
                  size="small"
                  type="text"
                  icon={<EditOutlined />}
                  onClick={() => handleEditMessage(msg.id, msg.content)}
                />
              </Tooltip>
            </div>
          )}

          {/* Branch navigation (if message has multiple branches) */}
          {msg.branches && msg.branches.length > 1 && (
            <div className="flex items-center gap-2 mt-2">
              <Button size="small" type="text" icon={<LeftOutlined />} />
              <Text type="secondary" className="text-xs">
                {msg.branch_index + 1} / {msg.branches.length}
              </Text>
              <Button size="small" type="text" icon={<RightOutlined />} />
            </div>
          )}
        </div>

        {isUser && (
          <Avatar
            size="small"
            icon={<UserOutlined />}
            className="flex-shrink-0"
            style={{ backgroundColor: appTheme.chatMessageUser }}
          />
        )}
      </div>
    )
  }

  // Empty state (no conversation loaded)
  if (!conversation && !searchParams.get('conversation')) {
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
                      <Text type="secondary">(Personal)</Text>
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
                {modelProviders.map(provider =>
                  provider.models.map(model => (
                    <Option
                      key={`${provider.id}:${model.id}`}
                      value={`${provider.id}:${model.id}`}
                    >
                      <Space>
                        <Text>{model.name}</Text>
                        <Text type="secondary">({provider.name})</Text>
                      </Space>
                    </Option>
                  )),
                )}
              </Select>
            </Col>
          </Row>
        </div>

        {/* Welcome message */}
        <div className="flex flex-col items-center justify-center flex-1 text-center p-8">
          <div className="mb-8">
            <div
              className="text-2xl font-bold mb-2"
              style={{ color: appTheme.primary }}
            >
              🌟 Welcome to Chat
            </div>
            <Text type="secondary">
              Select an assistant and model to start chatting
            </Text>
          </div>

          <div
            className="w-full max-w-2xl p-4 sm:p-6 rounded-xl border"
            style={{
              backgroundColor: appTheme.surfaceElevated,
              borderColor: appTheme.borderLight,
            }}
          >
            <TextArea
              value={inputValue}
              onChange={e => setInputValue(e.target.value)}
              onKeyPress={handleKeyPress}
              placeholder="How can I help you today?"
              className="w-full text-base bg-transparent border-none outline-none"
              style={{ color: appTheme.textPrimary }}
              autoSize={{ minRows: 1, maxRows: 4 }}
              disabled={!selectedAssistant || !selectedModel}
            />

            <Divider style={{ margin: '16px 0' }} />

            <div className="flex justify-between items-center">
              <div className="flex gap-2">
                <Text type="secondary" className="text-sm">
                  {selectedAssistant && selectedModel
                    ? 'Ready to chat'
                    : 'Select assistant and model'}
                </Text>
              </div>

              <div className="flex items-center gap-2">
                {selectedModel && (
                  <Text type="secondary" className="text-sm">
                    {selectedModel.split(':')[1]}
                  </Text>
                )}
                <Button
                  type="primary"
                  icon={<SendOutlined />}
                  onClick={handleSend}
                  disabled={
                    !inputValue.trim() || !selectedAssistant || !selectedModel
                  }
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

  return (
    <div
      className="flex flex-col h-full"
      style={{ backgroundColor: appTheme.chatBackground }}
    >
      {/* Header with conversation title and controls */}
      <div
        className="px-4 sm:px-6 py-4 border-b"
        style={{
          backgroundColor: appTheme.surface,
          borderColor: appTheme.borderSecondary,
        }}
      >
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <RobotOutlined
              className="text-xl"
              style={{ color: appTheme.primary }}
            />
            <Text strong className="text-base">
              {conversation?.title || 'Chat'}
            </Text>
          </div>

          <div className="flex items-center gap-2">
            <Text type="secondary" className="text-sm">
              {selectedAssistant &&
                assistants.find(a => a.id === selectedAssistant)?.name}
            </Text>
            <Text type="secondary" className="text-sm">
              •
            </Text>
            <Text type="secondary" className="text-sm">
              {selectedModel?.split(':')[1]}
            </Text>
          </div>
        </div>
      </div>

      {/* Messages */}
      <div className="flex-1 overflow-auto px-4 sm:px-6 py-4 flex flex-col">
        {messages.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-center">
            <MessageOutlined
              className="text-5xl mb-4"
              style={{ color: appTheme.textTertiary }}
            />
            <Text
              className="text-base"
              style={{ color: appTheme.textSecondary }}
            >
              Start your conversation
            </Text>
          </div>
        ) : (
          <>
            {messages.map(renderMessage)}
            {isLoading && (
              <div className="flex items-center gap-3 mb-4">
                <Avatar
                  size="small"
                  icon={<RobotOutlined />}
                  className="flex-shrink-0"
                  style={{ backgroundColor: appTheme.success }}
                />
                <Card
                  size="small"
                  className="border-none rounded-xl"
                  style={{ backgroundColor: appTheme.chatMessageAssistant }}
                  bodyStyle={{ padding: '8px 12px' }}
                >
                  <Spin
                    indicator={
                      <LoadingOutlined style={{ fontSize: 16 }} spin />
                    }
                  />
                  <Text type="secondary" className="ml-2">
                    Thinking...
                  </Text>
                </Card>
              </div>
            )}
            <div ref={messagesEndRef} />
          </>
        )}
      </div>

      {/* Input */}
      <div
        className="px-4 sm:px-6 py-4 border-t"
        style={{
          backgroundColor: appTheme.surface,
          borderColor: appTheme.borderSecondary,
        }}
      >
        <div className="flex items-end gap-2">
          <TextArea
            value={inputValue}
            onChange={e => setInputValue(e.target.value)}
            onKeyPress={handleKeyPress}
            placeholder="Type your message..."
            autoSize={{ minRows: 1, maxRows: 4 }}
            className="flex-1"
            disabled={isLoading}
          />
          <div className="flex gap-2">
            {isLoading && (
              <Button
                type="text"
                icon={<StopOutlined />}
                onClick={handleStopGeneration}
                danger
              >
                Stop
              </Button>
            )}
            <Button
              type="primary"
              icon={<SendOutlined />}
              onClick={handleSend}
              disabled={!inputValue.trim() || isLoading}
            >
              Send
            </Button>
          </div>
        </div>
      </div>
    </div>
  )
}
