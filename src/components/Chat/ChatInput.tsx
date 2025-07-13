import { memo, useState } from 'react'
import { Button, Flex, Input } from 'antd'
import { useTranslation } from 'react-i18next'
import { SendOutlined, StopOutlined } from '@ant-design/icons'

const { TextArea } = Input

interface ChatInputProps {
  onSend: (message: string) => void | Promise<void>
  onStop?: () => void
  disabled?: boolean
  isLoading?: boolean
  isStreaming?: boolean
  placeholder?: string
}

export const ChatInput = memo(function ChatInput({
  onSend,
  onStop,
  disabled = false,
  isLoading = false,
  isStreaming = false,
  placeholder = 'Message AI...',
}: ChatInputProps) {
  const { t } = useTranslation()
  const [inputValue, setInputValue] = useState('')
  const [isSending, setIsSending] = useState(false)

  const handleSend = async () => {
    if (!inputValue.trim() || disabled || isSending) return

    const messageToSend = inputValue.trim()
    setInputValue('') // Clear input immediately
    setIsSending(true)

    try {
      await onSend(messageToSend)
    } catch (error) {
      console.error('Failed to send message:', error)
      // Restore the message if sending failed
      setInputValue(messageToSend)
    } finally {
      setIsSending(false)
    }
  }

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      handleSend()
    }
  }

  const handleStop = () => {
    setIsSending(false)
    onStop?.()
  }

  const isDisabled = disabled || isSending
  const showStop = isLoading || isStreaming || isSending

  return (
    <Flex className="flex items-end gap-1 w-full">
      <div className="flex-1">
        <TextArea
          value={inputValue}
          onChange={e => setInputValue(e.target.value)}
          onKeyPress={handleKeyPress}
          placeholder={placeholder}
          autoSize={{ minRows: 1, maxRows: 6 }}
          disabled={isDisabled}
          className="resize-none"
        />
      </div>
      <div className="flex gap-2">
        {showStop && (
          <Button type="text" icon={<StopOutlined />} onClick={handleStop}>
            {t('chat.stop')}
          </Button>
        )}
        <Button
          type="primary"
          icon={<SendOutlined />}
          onClick={handleSend}
          disabled={!inputValue.trim() || isDisabled}
          loading={isSending}
        >
          {t('chat.send')}
        </Button>
      </div>
    </Flex>
  )
})
