import { useParams } from 'react-router-dom'
import { ChatInterface } from '../Chat/ChatInterface'

export function ChatPage() {
  const { conversationId } = useParams<{ conversationId?: string }>()

  return <ChatInterface conversationId={conversationId || null} />
}
