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
  match = 'all',
}: {
  permissions: Permission[]
  children: React.ReactNode
  type?: 'hidden' | 'disabled'
  match?: 'any' | 'all'
}) => {
  if (match === 'any') {
    if (permissions.some(p => hasPermission([p]))) {
      return children
    }
  } else {
    if (!hasPermission(permissions)) {
      if (type === 'hidden') {
        return null
      }
      return disableChildren(children)
    }
    return children
  }
}

export const PagePermissionGuard403 = ({
  permissions,
  children,
  match = 'all',
}: {
  permissions: Permission[]
  children: React.ReactNode
  match?: 'any' | 'all'
}) => {
  const navigate = useNavigate()
  let isGranted = false
  if (match === 'any') {
    isGranted = permissions.some(p => hasPermission([p]))
  } else {
    // Check if all permissions are granted
    isGranted = hasPermission(permissions)
  }
  if (!isGranted) {
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
