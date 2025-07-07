import {useState, useRef, useEffect} from 'react'
import {Input, Button, Space, Typography, Avatar, Card, Spin} from 'antd'
import {SendOutlined, UserOutlined, RobotOutlined, LoadingOutlined, MessageOutlined} from '@ant-design/icons'
import {useAppStore} from '../../store'
import {useTheme} from '../../hooks/useTheme'

const {TextArea} = Input
const {Text} = Typography

interface ChatInterfaceProps {
    threadId: string | null
}

export function ChatInterface({threadId}: ChatInterfaceProps) {
    const appTheme = useTheme()
    const {messages, addMessage, threads} = useAppStore()
    const [inputValue, setInputValue] = useState('')
    const [isLoading, setIsLoading] = useState(false)
    const messagesEndRef = useRef<HTMLDivElement>(null)

    const currentThread = threads.find(t => t.id === threadId)
    const threadMessages = messages.filter(m => m.threadId === threadId)

    useEffect(() => {
        scrollToBottom()
    }, [threadMessages])

    const scrollToBottom = () => {
        messagesEndRef.current?.scrollIntoView({behavior: 'smooth'})
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
                className={`flex ${isUser ? 'justify-end' : 'justify-start'} mb-4 gap-3`}
            >
                {isAssistant && (
                    <Avatar
                        size="small"
                        icon={<RobotOutlined/>}
                        className="flex-shrink-0"
                        style={{backgroundColor: appTheme.success}}
                    />
                )}

                <Card
                    size="small"
                    className="max-w-[85%] sm:max-w-[70%] border-none rounded-xl"
                    style={{
                        backgroundColor: isUser ? appTheme.chatMessageUser : appTheme.chatMessageAssistant,
                        color: isUser ? appTheme.chatMessageUserText : appTheme.chatMessageAssistantText,
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
                        icon={<UserOutlined/>}
                        className="flex-shrink-0"
                        style={{backgroundColor: appTheme.chatMessageUser}}
                    />
                )}
            </div>
        )
    }

    if (!threadId) {
        return (
            <div className="flex flex-col items-center justify-center h-full text-center p-8" style={{
                backgroundColor: appTheme.chatBackground,
                color: appTheme.textPrimary
            }}>
                <div className="mb-8">
                    <div className="text-2xl font-bold mb-2" style={{color: appTheme.primary}}>
                        🌟 Evening, Phi
                    </div>
                </div>

                <div className="w-full max-w-2xl p-4 sm:p-6 rounded-xl border" style={{
                    backgroundColor: appTheme.surfaceElevated,
                    borderColor: appTheme.borderLight
                }}>
                    <input
                        placeholder="How can I help you today?"
                        className="w-full p-4 text-base bg-transparent border-none outline-none"
                        style={{color: appTheme.textPrimary}}
                    />

                    <div className="flex justify-between items-center mt-4 pt-4 border-t" style={{borderColor: appTheme.borderLight}}>
                        <div className="flex gap-2">
                            <button className="px-3 py-2 bg-transparent border rounded-md text-sm cursor-pointer" style={{
                                borderColor: appTheme.border,
                                color: appTheme.textSecondary
                            }}>
                                + Research
                            </button>
                        </div>

                        <div className="flex items-center gap-2">
              <span className="text-sm" style={{color: appTheme.textTertiary}}>
                Claude Sonnet 4
              </span>
                            <button className="p-2 border-none rounded-md cursor-pointer" style={{
                                backgroundColor: appTheme.primary,
                                color: '#ffffff'
                            }}>
                                ↑
                            </button>
                        </div>
                    </div>
                </div>

                <div className="flex gap-2 sm:gap-4 mt-8 flex-wrap justify-center">
                    {[
                        {icon: '✍️', label: 'Write'},
                        {icon: '🎓', label: 'Learn'},
                        {icon: '</>', label: 'Code'},
                        {icon: '🎯', label: 'Life stuff'},
                        {icon: '🔗', label: 'Connect apps', badge: 'NEW'}
                    ].map((item, index) => (
                        <button
                            key={index}
                            className="flex items-center gap-2 px-3 sm:px-4 py-2 sm:py-3 bg-transparent border rounded-lg cursor-pointer text-sm relative"
                            style={{
                                borderColor: appTheme.border,
                                color: appTheme.textSecondary
                            }}
                        >
                            <span>{item.icon}</span>
                            <span>{item.label}</span>
                            {item.badge && (
                                <span className="text-xs px-1.5 py-0.5 rounded ml-1" style={{
                                    backgroundColor: appTheme.primary,
                                    color: '#ffffff'
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
        <div className="flex flex-col h-full" style={{backgroundColor: appTheme.chatBackground}}>
            {/* Header */}
            <div className="px-4 sm:px-6 py-4 border-b flex items-center gap-3" style={{
                backgroundColor: appTheme.surface,
                borderColor: appTheme.borderSecondary
            }}>
                <RobotOutlined className="text-xl" style={{color: appTheme.primary}}/>
                <Text strong className="text-base">
                    {currentThread?.title || 'Chat'}
                </Text>
            </div>

            {/* Messages */}
            <div className="flex-1 overflow-auto px-4 sm:px-6 py-4 flex flex-col">
                {threadMessages.length === 0 ? (
                    <div className="flex flex-col items-center justify-center h-full text-center">
                        <MessageOutlined className="text-5xl mb-4" style={{color: appTheme.textTertiary}}/>
                        <Text className="text-base" style={{color: appTheme.textSecondary}}>
                            Start your conversation
                        </Text>
                    </div>
                ) : (
                    <>
                        {threadMessages.map(renderMessage)}
                        {isLoading && (
                            <div className="flex items-center gap-3 mb-4">
                                <Avatar
                                    size="small"
                                    icon={<RobotOutlined/>}
                                    className="flex-shrink-0"
                                    style={{backgroundColor: appTheme.success}}
                                />
                                <Card
                                    size="small"
                                    className="border-none rounded-xl"
                                    style={{backgroundColor: appTheme.chatMessageAssistant}}
                                    bodyStyle={{padding: '8px 12px'}}
                                >
                                    <Spin indicator={<LoadingOutlined style={{fontSize: 16}} spin/>}/>
                                    <Text type="secondary" className="ml-2">
                                        Thinking...
                                    </Text>
                                </Card>
                            </div>
                        )}
                        <div ref={messagesEndRef}/>
                    </>
                )}
            </div>

            {/* Input */}
            <div className="px-4 sm:px-6 py-4 border-t" style={{
                backgroundColor: appTheme.surface,
                borderColor: appTheme.borderSecondary
            }}>
                <Space.Compact className="flex w-full">
                    <TextArea
                        value={inputValue}
                        onChange={(e) => setInputValue(e.target.value)}
                        onKeyPress={handleKeyPress}
                        placeholder="Type your message..."
                        autoSize={{minRows: 1, maxRows: 4}}
                        className="flex-1"
                        disabled={isLoading}
                    />
                    <Button
                        type="primary"
                        icon={<SendOutlined/>}
                        onClick={handleSend}
                        disabled={!inputValue.trim() || isLoading}
                        className="h-auto"
                    >
                        Send
                    </Button>
                </Space.Compact>
            </div>
        </div>
    )
}