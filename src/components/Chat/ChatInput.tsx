import { memo, useState } from 'react'
import { Button, Flex, Input } from 'antd'
import { useTranslation } from 'react-i18next'
import { SendOutlined, StopOutlined } from '@ant-design/icons'
import { useShallow } from 'zustand/react/shallow'
import { useChatStore } from '../../store/chat'

const { TextArea } = Input

interface ChatInputProps {
  onSend?: (message: string) => void | Promise<void>
  onStop?: () => void
  disabled?: boolean
  placeholder?: string
}

export const ChatInput = memo(function ChatInput({
  onSend,
  onStop,
  disabled: externalDisabled,
  placeholder,
}: ChatInputProps) {
  const { t } = useTranslation()
  const [inputValue, setInputValue] = useState('')

  const {
    currentConversation,
    sending,
    isStreaming,
    sendMessage,
    stopStreaming,
  } = useChatStore(
    useShallow(state => ({
      currentConversation: state.currentConversation,
      sending: state.sending,
      isStreaming: state.isStreaming,
      sendMessage: state.sendMessage,
      stopStreaming: state.stopStreaming,
    })),
  )

  const handleSend = async () => {
    const messageToSend = inputValue.trim()
    if (!messageToSend) return

    // If external onSend is provided (for new conversations), use it
    if (onSend) {
      if (externalDisabled) return
      setInputValue('') // Clear input immediately
      try {
        await onSend(messageToSend)
      } catch (error) {
        console.error('Failed to send message:', error)
        // Restore the message if sending failed
        setInputValue(messageToSend)
      }
      return
    }

    // Otherwise, use store for existing conversations
    if (sending || !currentConversation) return
    setInputValue('') // Clear input immediately

    try {
      await sendMessage(
        messageToSend,
        currentConversation.assistant_id,
        currentConversation.model_id,
      )
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

  const handleStop = () => {
    if (onStop) {
      onStop()
    } else {
      stopStreaming()
    }
  }

  // For external usage (new conversations)
  if (onSend) {
    const isDisabled = externalDisabled

    return (
      <Flex className="flex items-end gap-1 w-full">
        <div className="flex-1">
          <TextArea
            value={inputValue}
            onChange={e => setInputValue(e.target.value)}
            onKeyPress={handleKeyPress}
            placeholder={placeholder || t('chat.messageAI')}
            autoSize={{ minRows: 1, maxRows: 6 }}
            disabled={isDisabled}
            className="resize-none"
          />
        </div>
        <div className="flex gap-2">
          <Button
            type="primary"
            icon={<SendOutlined />}
            onClick={handleSend}
            disabled={!inputValue.trim() || isDisabled}
          >
            {t('chat.send')}
          </Button>
        </div>
      </Flex>
    )
  }

  // For internal usage (existing conversations)
  const isDisabled = sending || !currentConversation
  const showStop = sending || isStreaming

  return (
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
          <Button type="text" icon={<StopOutlined />} onClick={handleStop}>
            {t('chat.stop')}
          </Button>
        )}
        <Button
          type="primary"
          icon={<SendOutlined />}
          onClick={handleSend}
          disabled={!inputValue.trim() || isDisabled}
          loading={sending}
        >
          {t('chat.send')}
        </Button>
      </div>
    </Flex>
  )
})
