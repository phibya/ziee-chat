import { memo } from 'react'
import { Flex, theme, Typography } from 'antd'
import { Stores } from '../../store'

const { Text } = Typography

export const ChatHeader = memo(function ChatHeader() {
  const { currentConversation } = Stores.Chat
  const { token } = theme.useToken()

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
    </Flex>
  )
})
