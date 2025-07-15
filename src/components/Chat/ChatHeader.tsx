import { memo } from 'react'
import { Flex, theme, Typography } from 'antd'
import { useShallow } from 'zustand/react/shallow'
import { useChatStore } from '../../store/chat'
import { useAssistantsStore } from '../../store/assistants'
import { useModelProvidersStore } from '../../store/modelProviders'

const { Text } = Typography

export const ChatHeader = memo(function ChatHeader() {
  const { currentConversation } = useChatStore(
    useShallow(state => ({
      currentConversation: state.currentConversation,
    })),
  )

  const { assistants } = useAssistantsStore(
    useShallow(state => ({
      assistants: state.assistants,
    })),
  )

  const { token } = theme.useToken()

  const getModelDisplayName = () => {
    if (!currentConversation?.model_id) return ''

    const model = useModelProvidersStore
      .getState()
      .getModelById(currentConversation.model_id)
    return model?.alias || currentConversation.model_id
  }

  const getAssistantName = () => {
    if (!currentConversation?.assistant_id) return ''
    return (
      assistants.find(a => a.id === currentConversation.assistant_id)?.name ||
      ''
    )
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
          {currentConversation?.title || 'Claude'}
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
