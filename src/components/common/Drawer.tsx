import {
  Button,
  Drawer as AntDrawer,
  DrawerProps as AntDrawerProps,
  theme,
  Typography,
} from 'antd'
import React, { useEffect, useRef } from 'react'
import { ResizeHandle } from './ResizeHandle.tsx'
import tinycolor from 'tinycolor2'
import {isMacOS, isTauriView} from '../../api/core.ts'
import { useWindowMinSize } from '../hooks/useWindowMinSize.ts'
import { IoIosArrowBack } from 'react-icons/io'
import { TauriDragRegion } from './TauriDragRegion.tsx'

export interface DrawerProps extends AntDrawerProps {
  children?: React.ReactNode
}

export const Drawer: React.FC<DrawerProps> = props => {
  const { token } = theme.useToken()
  const windowMinSize = useWindowMinSize()

  const drawerDivRef = useRef<HTMLDivElement>(null)
  const titleRef = useRef<HTMLDivElement>(null)

  // Monitor the left position of the drawer div and adjust title padding
  useEffect(() => {
    if (!isTauriView) return
    if (!props.open) return

    console.log('Setting up ResizeObserver for drawer position monitoring')
    const monitorPosition = () => {
      if (drawerDivRef.current && titleRef.current) {
        const rect = drawerDivRef.current.getBoundingClientRect()
        const leftMin = isMacOS ? 72 : 0
        if (rect.left < leftMin) {
          titleRef.current.style.paddingLeft = leftMin - rect.left + 'px'
        } else {
          titleRef.current.style.paddingLeft = ''
        }
      }
    }

    const resizeObserver = new ResizeObserver(monitorPosition)

    if (drawerDivRef.current) {
      resizeObserver.observe(drawerDivRef.current)
    }

    setTimeout(() => {
      monitorPosition()
    }, 300)

    return () => {
      console.log('Disconnecting ResizeObserver')
      resizeObserver.disconnect()
    }
  }, [props.open])

  const {
    placement = 'right',
    width = 520,
    maskClosable = true,
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
        body: `!p-3 !pt-0`,
        wrapper: '!overflow-hidden !bg-transparent',
        ...(restProps.classNames || {}),
      }}
      title={
        props.title ? (
          <div
            ref={titleRef}
            className={
              'flex w-full items-center gap-1 py-2 pt-[10px] px-1 relative'
            }
          >
            <TauriDragRegion
              className={'h-full w-full absolute top-0 left-0'}
            />
            <Button
              type={'text'}
              onClick={props.onClose}
              style={{
                width: 30,
              }}
            >
              <div className={'text-xl'}>
                <IoIosArrowBack />
              </div>
            </Button>
            {typeof props.title === 'string' ? (
              <Typography.Title level={5} className={'!m-0'}>
                {props.title}
              </Typography.Title>
            ) : (
              props.title
            )}
          </div>
        ) : null
      }
      styles={{
        header: {
          borderBottom: 'none',
          padding: 0,
          ...(restProps.styles?.header || {}),
        },
        footer: {
          borderTop: 'none',
          padding: '6px 12px 12px 12px',
          ...(restProps.styles?.footer || {}),
        },
        mask: {
          backdropFilter: 'brightness(0.75)',
          backgroundColor: tinycolor(token.colorBgLayout)
            .setAlpha(0.75)
            .toString(),
          ...(restProps.styles?.mask || {}),
        },
        wrapper: {
          border:
            windowMinSize.xs && !isTauriView
              ? 'none'
              : `1px solid ${token.colorBorderSecondary}`,
          borderRadius: isTauriView ? 8 : windowMinSize.xs ? 0 : 8,
          maxWidth: `calc(100vw - ${isTauriView && windowMinSize.xs ? 0 : isTauriView ? 90 : windowMinSize.xs ? 0 : 24}px)`,
          boxShadow: 'none',
          margin: windowMinSize.xs ? 0 : 12,
          ...(restProps.styles?.wrapper || {}),
        },
        content: {
          backgroundColor: token.colorBgLayout,
          ...(restProps.styles?.content || {}),
        },
      }}
      // className={`!bg-transparent !m-3 ${className}`}
      drawerRender={node => {
        return (
          <div
            ref={drawerDivRef}
            className={'w-full h-full'}
            onTouchStart={e => e.stopPropagation()}
            onTouchMove={e => e.stopPropagation()}
            onTouchEnd={e => e.stopPropagation()}
            onScroll={e => e.stopPropagation()}
            onWheel={e => e.stopPropagation()}
          >
            <div className={'w-full h-full'}>{node}</div>
            <ResizeHandle placement={'left'} parentLevel={[1]} />
          </div>
        )
      }}
    />
  )
}
