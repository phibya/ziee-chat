import { useAppStore } from '../../store'
import { ChatInterface } from '../Chat/ChatInterface'

export function ChatPage() {
  const { currentThreadId } = useAppStore()

  return <ChatInterface threadId={currentThreadId} />
}
