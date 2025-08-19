import { useTranslation } from 'react-i18next'
import { ChatInput } from './ChatInput'
import { TauriDragRegion } from '../../common/TauriDragRegion.tsx'

export function NewChatInterface() {
  const { t } = useTranslation()

  return (
    <div className="flex flex-col h-full w-full">
      <TauriDragRegion className="h-[50px] w-full absolute top-0 left-0" />
      {/* Welcome message */}
      <div className="flex flex-col items-center justify-center flex-1 text-center p-3">
        <div className="mb-3">
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
