import { Flex, Typography } from 'antd'
import { ReactNode } from 'react'

const { Title, Text } = Typography

interface SettingsPageContainerProps {
  title: string
  subtitle?: string
  children: ReactNode
}

export function SettingsPageContainer({
  title,
  subtitle,
  children,
}: SettingsPageContainerProps) {
  return (
    <Flex className="p-3 flex-col gap-3">
      <Flex className="flex-col">
        <Title level={3} className="m-0 p-0">
          {title}
        </Title>
        {subtitle && (
          <Text type="secondary" className="text-base">
            {subtitle}
          </Text>
        )}
      </Flex>
      <Flex className={'flex-col gap-3'}>{children}</Flex>
    </Flex>
  )
}
