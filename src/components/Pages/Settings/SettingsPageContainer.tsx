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
      <div className="w-full flex justify-center pt-3 !px-3">
        <div className={'max-w-6xl w-full flex flex-col gap-0'}>
          <Title level={3} className="!m-0 !p-0 !leading-tight">
            {title}
          </Title>
          {subtitle && (
            <Text
              type="secondary"
              className=" !m-0 !p-0 text-base !leading-tight"
            >
              {subtitle}
            </Text>
          )}
        </div>
      </div>
      <div className={'flex-1 w-full overflow-auto flex justify-center'}>
        <div className={'max-w-6xl w-full gap-3 !px-3'}>
          {children}
          <div className="h-3" />
        </div>
      </div>
    </Flex>
  )
}
