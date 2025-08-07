import { Drawer as AntDrawer, DrawerProps as AntDrawerProps, theme } from 'antd'
import React from 'react'
import { ResizeHandle } from './ResizeHandle.tsx'
import tinycolor from 'tinycolor2'
import { isDesktopApp } from '../../api/core.ts'
import { useWindowMinSize } from '../hooks/useWindowMinSize.ts'

export interface DrawerProps extends AntDrawerProps {
  children?: React.ReactNode
}

export const Drawer: React.FC<DrawerProps> = props => {
  const { token } = theme.useToken()
  const windowMinSize = useWindowMinSize()

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
      width={windowMinSize.xs ? '100%' : width}
      maskClosable={maskClosable}
      {...restProps}
      closable={false}
      classNames={{
        wrapper: '!overflow-hidden !bg-transparent',
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
          backdropFilter: 'brightness(0.7)',
          backgroundColor: tinycolor(token.colorBgLayout)
            .setAlpha(0.7)
            .toString(),
        },
        wrapper: {
          border: `1px solid ${token.colorBorder}`,
          borderRadius: isDesktopApp ? 8 : windowMinSize.xs ? 0 : 8,
          maxWidth: `calc(100vw - ${isDesktopApp ? 90 : windowMinSize.xs ? 0 : 24}px)`,
          boxShadow: 'none',
          margin: windowMinSize.xs ? 0 : 12,
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
