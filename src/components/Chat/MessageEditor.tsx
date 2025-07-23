import { memo } from 'react'
import { Button, Input } from 'antd'
import { useTranslation } from 'react-i18next'
import { CloseOutlined, SaveOutlined } from '@ant-design/icons'
import { editChatMessage } from '../../store'
import {
  useChatUIStore,
  updateEditingContent,
  stopEditingMessage,
} from '../../store/ui/chat'

const { TextArea } = Input

export const MessageEditor = memo(function MessageEditor() {
  const { t } = useTranslation()

  const { editingMessageId, editingMessageContent } = useChatUIStore()

  // External editMessage function is imported from store

  const handleSave = async () => {
    if (!editingMessageId) return
    try {
      await editChatMessage(editingMessageId, editingMessageContent)
      stopEditingMessage()
    } catch (error) {
      console.error('Failed to save edit:', error)
      stopEditingMessage()
    }
  }

  const handleCancel = () => {
    stopEditingMessage()
  }

  return (
    <div className={'flex flex-col gap-1 w-full'}>
      <TextArea
        value={editingMessageContent}
        onChange={e => updateEditingContent(e.target.value)}
        autoSize={{ minRows: 2, maxRows: 8 }}
        className="mb-2 w-full"
      />
      <div className="flex gap-1">
        <Button
          size="small"
          type="primary"
          icon={<SaveOutlined />}
          onClick={handleSave}
        >
          {t('chat.save')}
        </Button>
        <Button size="small" icon={<CloseOutlined />} onClick={handleCancel}>
          {t('chat.cancel')}
        </Button>
      </div>
    </div>
  )
})
