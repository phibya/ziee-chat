import { useParams } from 'react-router-dom'
import { NewChatInterface } from './NewChatInterface'
import { ExistingChatInterface } from './ExistingChatInterface'

export function ChatInterface() {
  const { conversationId } = useParams<{ conversationId?: string }>()

  // Route between new chat and existing chat interfaces
  if (!conversationId) {
    return <NewChatInterface />
  }

  return <ExistingChatInterface />
}
