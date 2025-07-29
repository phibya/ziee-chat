import React from 'react'

interface PageContainerProps {
  children: React.ReactNode
}

export const PageContainer: React.FC<PageContainerProps> = ({ children }) => {
  return (
    <div className="flex flex-col overflow-hidden pt-2 px-3 pb-0 h-dvh w-full justify-items-center">
      <div className={'max-w-6xl w-full h-full flex-1 self-center'}>
        <div className={'w-full h-full'}>{children}</div>
      </div>
    </div>
  )
}
