import { Drawer as AntDrawer, DrawerProps as AntDrawerProps } from 'antd'
import React from 'react'

export interface DrawerProps extends AntDrawerProps {
  children?: React.ReactNode
}

export const Drawer: React.FC<DrawerProps> = props => {
  const {
    placement = 'right',
    width = 520,
    maskClosable = true,
    destroyOnHidden = true,
    ...restProps
  } = props

  return (
    <AntDrawer
      placement={placement}
      width={width}
      maskClosable={maskClosable}
      destroyOnHidden={destroyOnHidden}
      {...restProps}
    />
  )
}
