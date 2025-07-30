import { useTranslation } from 'react-i18next'
import { ChatInput } from './ChatInput'

export function NewChatInterface() {
  const { t } = useTranslation()

  return (
    <div className="flex flex-col h-full">
      {/* Welcome message */}
      <div className="flex flex-col items-center justify-center flex-1 text-center p-8">
        <div className="mb-8">
          <div className="text-3xl font-light mb-4">
            {t('chat.placeholderWelcome')}
          </div>
        </div>

        <div className="w-full max-w-2xl">
          <ChatInput />
        </div>
      </div>
    </div>
  )
}
