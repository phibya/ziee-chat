import { useParams } from 'react-router-dom'
import { ChatInterface } from '../Chat/ChatInterface'
import { PageContainer } from '../common/PageContainer'

export function ChatPage() {
  const { conversationId } = useParams<{ conversationId?: string }>()

  return (
    <PageContainer>
      <div className="h-full">
        <ChatInterface conversationId={conversationId || null} />
      </div>
    </PageContainer>
  )
}
