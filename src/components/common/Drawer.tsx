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
      styles={{
        body: {
          padding: 12,
        },
        header: {
          borderBottom: 'none',
          padding: '12px 12px 6px 12px',
          zIndex: 1002,
        },
        footer: {
          borderTop: 'none',
          padding: '6px 12px 12px 12px',
        },
        mask: {
          backdropFilter: 'blur(5px)',
        },
      }}
      style={{
        borderRadius: '8px 0 0 8px',
        border: `1px solid ${token.colorBorder}`,
        borderRight: 'none',
      }}
      drawerRender={node => {
        return (
          <div className={'w-full h-full relative flex flex-col'}>
            {node}
            <ResizeHandle placement={'left'} parentLevel={[1]} />
          </div>
        )
      }}
    />
  )
}
