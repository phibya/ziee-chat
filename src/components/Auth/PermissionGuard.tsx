import React from 'react'
import { Permission } from '../../types'
import { hasPermission, disableChildren } from '../../permissions/utils'
import { Button, Result } from 'antd'
import { useNavigate } from 'react-router-dom'
import { CiLock } from 'react-icons/ci'

export const PermissionGuard = ({
  permissions,
  children,
  type = 'hidden',
}: {
  permissions: Permission[]
  children: React.ReactNode
  type?: 'hidden' | 'disabled'
}) => {
  if (!hasPermission(permissions)) {
    if (type === 'hidden') {
      return null
    }
    // Properly disable all children components
    return disableChildren(children)
  }
  return children
}

export const PagePermissionGuard403 = ({
  permissions,
  children,
}: {
  permissions: Permission[]
  children: React.ReactNode
}) => {
  const navigate = useNavigate()
  if (!hasPermission(permissions)) {
    return (
      <div className={'w-full h-full flex items-center justify-center'}>
        <Result
          icon={
            <div className={'w-full flex items-center justify-center text-8xl'}>
              <CiLock />
            </div>
          }
          title="Permission Denied"
          subTitle="You do not have permission to access this resource."
          extra={
            <Button type="primary" onClick={() => navigate('/')}>
              Go to Home
            </Button>
          }
        />
      </div>
    )
  }
  return children
}
