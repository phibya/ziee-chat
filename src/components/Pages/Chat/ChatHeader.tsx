import { memo } from 'react'
import { Flex, theme, Typography } from 'antd'
import { useChatStore } from '../../../store'

const { Text } = Typography

export const ChatHeader = memo(function ChatHeader() {
  const { conversation } = useChatStore()
  const { token } = theme.useToken()

  return (
    <Flex
      className="items-center justify-between w-full !px-3 !py-2"
      style={{
        borderBottom: `1px solid ${token.colorBorderSecondary}`,
      }}
    >
      <div className="flex items-center gap-3">
        <Text strong ellipsis>
          {conversation?.title || 'Untitled Conversation'}
        </Text>
      </div>
    </Flex>
  )
})
