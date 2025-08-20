import { User } from '../types'
import { PermissionKey } from './constants.ts'

/**
 * Permission checking result types
 */
export interface PermissionCheckResult {
  hasPermission: boolean
  reason?: string
}

/**
 * Permission context for components
 */
export interface PermissionContext {
  user: User | null
  effectivePermissions: PermissionKey[]
  hasPermission: (permission: PermissionKey) => boolean
  hasAnyPermission: (permissions: PermissionKey[]) => boolean
  hasAllPermissions: (permissions: PermissionKey[]) => boolean
}
