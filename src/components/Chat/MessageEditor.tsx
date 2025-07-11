import { memo } from 'react'
import { Button, Input } from 'antd'
import { useTranslation } from 'react-i18next'
import { CloseOutlined, SaveOutlined } from '@ant-design/icons'

const { TextArea } = Input

interface MessageEditorProps {
  value: string
  onChange: (value: string) => void
  onSave: () => void
  onCancel: () => void
}

export const MessageEditor = memo(function MessageEditor({
  value,
  onChange,
  onSave,
  onCancel,
}: MessageEditorProps) {
  const { t } = useTranslation()

  return (
    <div>
      <TextArea
        value={value}
        onChange={e => onChange(e.target.value)}
        autoSize={{ minRows: 2, maxRows: 8 }}
        className="mb-2"
      />
      <div className="flex gap-2">
        <Button
          size="small"
          type="primary"
          icon={<SaveOutlined />}
          onClick={onSave}
        >
          {t('chat.save')}
        </Button>
        <Button size="small" icon={<CloseOutlined />} onClick={onCancel}>
          {t('chat.cancel')}
        </Button>
      </div>
    </div>
  )
})
