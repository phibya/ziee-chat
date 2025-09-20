import { Stores } from '../../store'
import {isMacOS, isTauriView} from '../../api/core.ts'
import { theme } from 'antd'

interface TitleBarWrapperProps {
  children?: React.ReactNode
  className?: string
  style?: React.CSSProperties
}

export const TitleBarWrapper = ({
  children,
  className = '',
  style = {},
}: TitleBarWrapperProps) => {
  const { token } = theme.useToken()
  const { isSidebarCollapsed, isFullscreen } = Stores.UI.Layout
  return (
    <div
      className={`h-[50px] w-full flex relative border-b px-3 transition-all duration-200 ease-in-out box-border py-0 ${className}`}
      style={{
        paddingLeft:
          isSidebarCollapsed && isTauriView && !isFullscreen && isMacOS
            ? 110
            : isSidebarCollapsed
              ? 48
              : 12,
        paddingRight: isTauriView && !isFullscreen && !isMacOS ? 100 : 0,
        borderColor: token.colorBorderSecondary,
        backgroundColor: token.colorBgLayout,
        ...style,
      }}
    >
      {children}
    </div>
  )
}
