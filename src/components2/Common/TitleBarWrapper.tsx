import { Stores } from '../../store'
import { isDesktopApp } from '../../api/core.ts'

interface TitleBarWrapperProps {
  children?: React.ReactNode
  className?: string
}

export const TitleBarWrapper = ({
  children,
  className = '',
}: TitleBarWrapperProps) => {
  const { isSidebarCollapsed } = Stores.UI.Layout
  return (
    <div
      className={`h-[50px] w-full flex relative border-b border-gray-200 px-3 ${className} transition-all duration-200 ease-in-out box-border py-0`}
      style={{
        paddingLeft:
          isSidebarCollapsed && isDesktopApp
            ? 128
            : isSidebarCollapsed
              ? 48
              : 12,
      }}
    >
      {children}
    </div>
  )
}
