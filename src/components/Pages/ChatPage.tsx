import { useParams } from 'react-router-dom'
import { ChatInterface } from '../Chat/ChatInterface'

export function ChatPage() {
  const { conversationId } = useParams<{ conversationId?: string }>()

  return (
    <div className="flex justify-center h-full">
      <div className="w-full max-w-4xl">
        <ChatInterface conversationId={conversationId || null} />
      </div>
    </div>
  )
}
