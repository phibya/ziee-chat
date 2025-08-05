import React from 'react'
import { Typography } from 'antd'

interface PageContainerProps {
  children: React.ReactNode
  title?: string
  subtitle?: string
  extra?: React.ReactNode
}

export const PageContainer: React.FC<PageContainerProps> = ({
  children,
  title,
  subtitle,
  extra,
}) => {
  return (
    <div className="flex flex-col pb-0 h-dvh w-full justify-items-center">
      <div
        className={
          'flex max-w-6xl w-full self-center pt-3 pb-2 px-3 flex-wrap '
        }
      >
        <div className="flex justify-between items-center w-full flex-wrap">
          <div className={'flex flex-col gap-0 flex-[1_1_300px]'}>
            {title && (
              <Typography.Title level={2} className={'!leading-tight !mb-2'}>
                {title}
              </Typography.Title>
            )}
            {subtitle && (
              <Typography.Text type={'secondary'} className={'!leading-tight'}>
                {subtitle} asd
              </Typography.Text>
            )}
          </div>
          {!!extra && <div className={'flex-[1_1_200px]'}>{extra}</div>}
        </div>
      </div>
      <div
        className={
          'w-full flex-1 justify-items-center overflow-x-visible overflow-y-hidden'
        }
      >
        {children}
      </div>
    </div>
  )
}
