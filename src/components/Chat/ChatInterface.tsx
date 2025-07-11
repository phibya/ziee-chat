import { NewChatInterface } from './NewChatInterface'
import { ExistingChatInterface } from './ExistingChatInterface'

interface ChatInterfaceProps {
  conversationId: string | null
}

export function ChatInterface({ conversationId }: ChatInterfaceProps) {
  // Route between new chat and existing chat interfaces
  if (!conversationId) {
    return <NewChatInterface />
  }

  return <ExistingChatInterface conversationId={conversationId} />
}
