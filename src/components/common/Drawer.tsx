import { Drawer as AntDrawer, DrawerProps as AntDrawerProps, theme } from 'antd'
import React from 'react'
import { ResizeHandle } from './ResizeHandle.tsx'

export interface DrawerProps extends AntDrawerProps {
  children?: React.ReactNode
}

export const Drawer: React.FC<DrawerProps> = props => {
  const { token } = theme.useToken()

  const {
    placement = 'right',
    width = 520,
    maskClosable = true,
    className = '',
    ...restProps
  } = props

  if (Array.isArray(restProps.footer)) {
    restProps.footer = (
      <div className="flex gap-2">
        {restProps.footer.map((item, index) => (
          <React.Fragment key={index}>{item}</React.Fragment>
        ))}
      </div>
    )
  }

  return (
    <AntDrawer
      placement={placement}
      width={width}
      maskClosable={maskClosable}
      {...restProps}
      closable={false}
      classNames={{
        wrapper: '!m-3 !rounded-lg !overflow-hidden !bg-transparent',
      }}
      styles={{
        header: {
          borderBottom: 'none',
          padding: '12px 12px 6px 12px',
        },
        footer: {
          borderTop: 'none',
          padding: '6px 12px 12px 12px',
        },
        mask: {
          backdropFilter: 'blur(5px)',
        },
        wrapper: {
          border: `1px solid ${token.colorBorder}`,
          maxWidth: `calc(100vw - 1.5rem)`,
        },
      }}
      // className={`!bg-transparent !m-3 ${className}`}
      drawerRender={node => {
        return (
          <div className={'w-full h-full'}>
            {node}
            <ResizeHandle placement={'left'} parentLevel={[1]} />
          </div>
        )
      }}
    />
  )
}
