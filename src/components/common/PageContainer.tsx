import React from 'react'

interface PageContainerProps {
  children: React.ReactNode
}

export const PageContainer: React.FC<PageContainerProps> = ({ children }) => {
  return (
    <div className="flex flex-col overflow-auto pt-2 px-3 h-full w-full justify-items-center">
      <div className={'max-w-6xl w-full flex-1 self-center'}>
        <div className={'w-full h-full'}>{children}</div>
      </div>
      {/* Spacer to ensure content doesn't stick to the bottom */}
      <div className={'h-3 w-full'} />
    </div>
  )
}
