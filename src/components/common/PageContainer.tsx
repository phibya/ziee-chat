import React from 'react'

interface PageContainerProps {
  children: React.ReactNode
}

export const PageContainer: React.FC<PageContainerProps> = ({ children }) => {
  return (
    <div className="p-6 flex justify-center h-full w-full">
      <div className="w-full max-w-6xl">{children}</div>
    </div>
  )
}
