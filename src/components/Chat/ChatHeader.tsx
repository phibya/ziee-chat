import { memo } from 'react'
import { Flex, theme, Typography } from 'antd'
import { Conversation } from '../../types/api/chat'
import { Assistant } from '../../types/api/assistant'
import { ModelProvider } from '../../types/api/modelProvider'

const { Text } = Typography

interface ChatHeaderProps {
  conversation: Conversation | null
  selectedAssistant: string | null
  selectedModel: string | null
  assistants: Assistant[]
  modelProviders: ModelProvider[]
}

export const ChatHeader = memo(function ChatHeader({
  conversation,
  selectedAssistant,
  selectedModel,
  assistants,
  modelProviders,
}: ChatHeaderProps) {
  const { token } = theme.useToken()
  const getModelDisplayName = () => {
    if (!selectedModel) return ''

    const [providerId, modelId] = selectedModel.split(':')
    const provider = modelProviders.find(p => p.id === providerId)
    const model = provider?.models.find(m => m.id === modelId)
    return model?.alias || modelId
  }

  const getAssistantName = () => {
    if (!selectedAssistant) return ''
    return assistants.find(a => a.id === selectedAssistant)?.name || ''
  }

  return (
    <Flex
      className="items-center justify-between w-full !p-3"
      style={{
        borderBottom: `1px solid ${token.colorBorderSecondary}`,
      }}
    >
      <div className="flex items-center gap-3">
        <Text strong className="text-lg" ellipsis>
          {conversation?.title || 'Claude'}
        </Text>
      </div>

      <div className="flex items-center gap-2 text-sm">
        <span>{getAssistantName()}</span>
        <span>•</span>
        <span>{getModelDisplayName()}</span>
      </div>
    </Flex>
  )
})
