import { memo } from 'react'
import { Button, Flex, Input } from 'antd'
import { useTranslation } from 'react-i18next'
import { SendOutlined, StopOutlined } from '@ant-design/icons'

const { TextArea } = Input

interface ChatInputProps {
  value: string
  onChange: (value: string) => void
  onSend: () => void
  onStop: () => void
  onKeyPress: (e: React.KeyboardEvent) => void
  disabled: boolean
  isLoading: boolean
  isStreaming: boolean
  placeholder?: string
}

export const ChatInput = memo(function ChatInput({
  value,
  onChange,
  onSend,
  onStop,
  onKeyPress,
  disabled,
  isLoading,
  isStreaming,
  placeholder = 'Message Claude...',
}: ChatInputProps) {
  const { t } = useTranslation()

  return (
    <Flex className="flex items-end gap-1 w-full">
      <div className="flex-1">
        <TextArea
          value={value}
          onChange={e => onChange(e.target.value)}
          onKeyPress={onKeyPress}
          placeholder={placeholder}
          autoSize={{ minRows: 1, maxRows: 6 }}
          disabled={disabled}
          className="resize-none"
        />
      </div>
      <div className="flex gap-2">
        {(isLoading || isStreaming) && (
          <Button type="text" icon={<StopOutlined />} onClick={onStop}>
            {t('chat.stop')}
          </Button>
        )}
        <Button
          type="primary"
          icon={<SendOutlined />}
          onClick={onSend}
          disabled={!value.trim() || disabled}
        >
          {t('chat.send')}
        </Button>
      </div>
    </Flex>
  )
})
