import { useState, useRef, useEffect } from 'react'
import { Input, Button, Space, Typography, Avatar, List, Card, Spin } from 'antd'
import { SendOutlined, UserOutlined, RobotOutlined, LoadingOutlined, MessageOutlined } from '@ant-design/icons'
import { useAppStore } from '../../store'

const { TextArea } = Input
const { Text } = Typography

interface ChatInterfaceProps {
  threadId: string | null
}

export function ChatInterface({ threadId }: ChatInterfaceProps) {
  const { messages, addMessage, threads } = useAppStore()
  const [inputValue, setInputValue] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const messagesEndRef = useRef<HTMLDivElement>(null)

  const currentThread = threads.find(t => t.id === threadId)
  const threadMessages = messages.filter(m => m.threadId === threadId)

  useEffect(() => {
    scrollToBottom()
  }, [threadMessages])

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }

  const handleSend = async () => {
    if (!inputValue.trim() || !threadId) return

    const userMessage = {
      content: inputValue.trim(),
      role: 'user' as const,
      threadId,
    }

    addMessage(userMessage)
    setInputValue('')
    setIsLoading(true)

    // Simulate AI response
    setTimeout(() => {
      const aiMessage = {
        content: `I received your message: "${userMessage.content}". This is a simulated response from the AI assistant. In a real implementation, this would be connected to your LLM backend.`,
        role: 'assistant' as const,
        threadId,
      }
      addMessage(aiMessage)
      setIsLoading(false)
    }, 1000)
  }

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      handleSend()
    }
  }

  const renderMessage = (message: any) => {
    const isUser = message.role === 'user'
    const isAssistant = message.role === 'assistant'

    return (
      <div
        key={message.id}
        style={{
          display: 'flex',
          justifyContent: isUser ? 'flex-end' : 'flex-start',
          marginBottom: '16px',
          gap: '12px',
        }}
      >
        {isAssistant && (
          <Avatar
            size="small"
            icon={<RobotOutlined />}
            style={{ backgroundColor: '#52c41a', flexShrink: 0 }}
          />
        )}
        
        <Card
          size="small"
          style={{
            maxWidth: '70%',
            backgroundColor: isUser ? '#1890ff' : '#f5f5f5',
            color: isUser ? 'white' : 'inherit',
            border: 'none',
            borderRadius: '12px',
          }}
          bodyStyle={{
            padding: '8px 12px',
            whiteSpace: 'pre-wrap',
            wordBreak: 'break-word',
          }}
        >
          {message.content}
        </Card>

        {isUser && (
          <Avatar
            size="small"
            icon={<UserOutlined />}
            style={{ backgroundColor: '#1890ff', flexShrink: 0 }}
          />
        )}
      </div>
    )
  }

  if (!threadId) {
    return (
      <div style={{ 
        display: 'flex', 
        flexDirection: 'column', 
        alignItems: 'center', 
        justifyContent: 'center', 
        height: '100%',
        textAlign: 'center',
        padding: '32px',
        backgroundColor: 'rgb(26, 26, 26)',
        color: 'white'
      }}>
        <div style={{ marginBottom: '32px' }}>
          <div style={{ fontSize: '24px', fontWeight: 'bold', marginBottom: '8px', color: '#ff8c00' }}>
            🌟 Evening, Phi
          </div>
        </div>
        
        <div style={{ 
          width: '100%', 
          maxWidth: '600px',
          padding: '24px',
          backgroundColor: 'rgb(32, 32, 32)',
          borderRadius: '12px',
          border: '1px solid rgba(255,255,255,0.1)'
        }}>
          <input
            placeholder="How can I help you today?"
            style={{
              width: '100%',
              padding: '16px',
              fontSize: '16px',
              backgroundColor: 'transparent',
              border: 'none',
              outline: 'none',
              color: 'white',
              '::placeholder': { color: 'rgba(255,255,255,0.5)' }
            }}
          />
          
          <div style={{
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
            marginTop: '16px',
            paddingTop: '16px',
            borderTop: '1px solid rgba(255,255,255,0.1)'
          }}>
            <div style={{ display: 'flex', gap: '8px' }}>
              <button style={{
                padding: '8px 12px',
                backgroundColor: 'transparent',
                border: '1px solid rgba(255,255,255,0.2)',
                borderRadius: '6px',
                color: 'rgba(255,255,255,0.7)',
                fontSize: '14px',
                cursor: 'pointer'
              }}>
                + Research
              </button>
            </div>
            
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
              <span style={{ fontSize: '14px', color: 'rgba(255,255,255,0.5)' }}>
                Claude Sonnet 4
              </span>
              <button style={{
                padding: '8px',
                backgroundColor: '#ff8c00',
                border: 'none',
                borderRadius: '6px',
                color: 'white',
                cursor: 'pointer'
              }}>
                ↑
              </button>
            </div>
          </div>
        </div>

        <div style={{
          display: 'flex',
          gap: '16px',
          marginTop: '32px',
          flexWrap: 'wrap',
          justifyContent: 'center'
        }}>
          {[
            { icon: '✍️', label: 'Write' },
            { icon: '🎓', label: 'Learn' },
            { icon: '</>', label: 'Code' },
            { icon: '🎯', label: 'Life stuff' },
            { icon: '🔗', label: 'Connect apps', badge: 'NEW' }
          ].map((item, index) => (
            <button
              key={index}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '8px',
                padding: '12px 16px',
                backgroundColor: 'transparent',
                border: '1px solid rgba(255,255,255,0.2)',
                borderRadius: '8px',
                color: 'rgba(255,255,255,0.7)',
                cursor: 'pointer',
                fontSize: '14px',
                position: 'relative'
              }}
            >
              <span>{item.icon}</span>
              <span>{item.label}</span>
              {item.badge && (
                <span style={{
                  fontSize: '10px',
                  backgroundColor: '#ff8c00',
                  color: 'white',
                  padding: '2px 6px',
                  borderRadius: '4px',
                  marginLeft: '4px'
                }}>
                  {item.badge}
                </span>
              )}
            </button>
          ))}
        </div>
      </div>
    )
  }

  return (
    <div style={{ 
      display: 'flex', 
      flexDirection: 'column', 
      height: '100%',
      backgroundColor: '#fafafa',
    }}>
      {/* Header */}
      <div style={{ 
        padding: '16px 24px', 
        borderBottom: '1px solid #f0f0f0',
        backgroundColor: 'white',
        display: 'flex',
        alignItems: 'center',
        gap: '12px',
      }}>
        <RobotOutlined style={{ fontSize: '20px', color: '#1890ff' }} />
        <Text strong style={{ fontSize: '16px' }}>
          {currentThread?.title || 'Chat'}
        </Text>
      </div>

      {/* Messages */}
      <div style={{ 
        flex: 1, 
        overflow: 'auto', 
        padding: '16px 24px',
        display: 'flex',
        flexDirection: 'column',
      }}>
        {threadMessages.length === 0 ? (
          <div style={{ 
            display: 'flex', 
            flexDirection: 'column', 
            alignItems: 'center', 
            justifyContent: 'center', 
            height: '100%',
            textAlign: 'center',
          }}>
            <MessageOutlined style={{ fontSize: '48px', color: '#d9d9d9', marginBottom: '16px' }} />
            <Text type="secondary" style={{ fontSize: '16px' }}>
              Start your conversation
            </Text>
          </div>
        ) : (
          <>
            {threadMessages.map(renderMessage)}
            {isLoading && (
              <div style={{ display: 'flex', alignItems: 'center', gap: '12px', marginBottom: '16px' }}>
                <Avatar
                  size="small"
                  icon={<RobotOutlined />}
                  style={{ backgroundColor: '#52c41a', flexShrink: 0 }}
                />
                <Card
                  size="small"
                  style={{
                    backgroundColor: '#f5f5f5',
                    border: 'none',
                    borderRadius: '12px',
                  }}
                  bodyStyle={{ padding: '8px 12px' }}
                >
                  <Spin indicator={<LoadingOutlined style={{ fontSize: 16 }} spin />} />
                  <Text type="secondary" style={{ marginLeft: '8px' }}>
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
      <div style={{ 
        padding: '16px 24px', 
        borderTop: '1px solid #f0f0f0',
        backgroundColor: 'white',
      }}>
        <Space.Compact style={{ display: 'flex', width: '100%' }}>
          <TextArea
            value={inputValue}
            onChange={(e) => setInputValue(e.target.value)}
            onKeyPress={handleKeyPress}
            placeholder="Type your message..."
            autoSize={{ minRows: 1, maxRows: 4 }}
            style={{ flex: 1 }}
            disabled={isLoading}
          />
          <Button
            type="primary"
            icon={<SendOutlined />}
            onClick={handleSend}
            disabled={!inputValue.trim() || isLoading}
            style={{ height: 'auto' }}
          >
            Send
          </Button>
        </Space.Compact>
      </div>
    </div>
  )
}