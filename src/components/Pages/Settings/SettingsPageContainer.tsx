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
    <Flex className="p-3 flex-col gap-3 h-full overflow-hidden">
      <Flex className="flex-col gap-0">
        <Title level={3} className="!m-0 !p-0 !leading-tight">
          {title}
        </Title>
        {subtitle && (
          <Text
            type="secondary"
            className=" !m-0 !p-0 text-base !leading-tight"
          >
            {subtitle} ashgdahjs
          </Text>
        )}
      </Flex>
      <Flex className={'flex-col gap-3 flex-1 overflow-auto !py-3'}>
        {children}
      </Flex>
    </Flex>
  )
}
