import React from 'react'
import { Permission } from '../../types'
import { hasPermission, disableChildren } from '../../permissions/utils'

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
