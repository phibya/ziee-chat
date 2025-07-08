/**
 * Permission-related React hooks
 */

import { useMemo } from 'react'
import { useAuthStore } from '../store/auth'
import {
  canPerformAction,
  expandWildcardPermission,
  formatPermissionsForDisplay,
  getAllPermissionsGrouped,
  getMissingPermissions,
  getPermissionDescription,
  getPermissionDisplayName,
  getUserEffectivePermissions,
  groupPermissionsByCategory,
  hasAllPermissions,
  hasAnyPermission,
  hasPermission,
  isWildcardPermission,
} from './utils'
import type { PermissionKey } from './constants'
import type { PermissionContext } from './types'

/**
 * Main permission hook providing all permission-related functionality
 */
export function usePermissions(): PermissionContext {
  const user = useAuthStore(state => state.user)

  const effectivePermissions = useMemo(
    () => getUserEffectivePermissions(user),
    [user],
  )

  return {
    user,
    effectivePermissions,
    hasPermission: (permission: PermissionKey) =>
      hasPermission(user, permission),
    hasAnyPermission: (permissions: PermissionKey[]) =>
      hasAnyPermission(user, permissions),
    hasAllPermissions: (permissions: PermissionKey[]) =>
      hasAllPermissions(user, permissions),
  }
}

/**
 * Hook for checking a specific permission
 */
export function useHasPermission(permission: PermissionKey): boolean {
  const user = useAuthStore(state => state.user)
  return useMemo(() => hasPermission(user, permission), [user, permission])
}

/**
 * Hook for checking if user has any of the specified permissions
 */
export function useHasAnyPermission(permissions: PermissionKey[]): boolean {
  const user = useAuthStore(state => state.user)
  return useMemo(() => hasAnyPermission(user, permissions), [user, permissions])
}

/**
 * Hook for checking if user has all of the specified permissions
 */
export function useHasAllPermissions(permissions: PermissionKey[]): boolean {
  const user = useAuthStore(state => state.user)
  return useMemo(
    () => hasAllPermissions(user, permissions),
    [user, permissions],
  )
}

/**
 * Hook for getting user's effective permissions
 */
export function useEffectivePermissions(): PermissionKey[] {
  const user = useAuthStore(state => state.user)
  return useMemo(() => getUserEffectivePermissions(user), [user])
}

/**
 * Hook for getting permissions grouped by category
 */
export function usePermissionsByCategory(): Record<string, PermissionKey[]> {
  const effectivePermissions = useEffectivePermissions()
  return useMemo(
    () => groupPermissionsByCategory(effectivePermissions),
    [effectivePermissions],
  )
}

/**
 * Hook for checking if user can perform an action
 */
export function useCanPerformAction(
  requiredPermissions: PermissionKey[],
  requireAll: boolean = false,
): boolean {
  const user = useAuthStore(state => state.user)
  return useMemo(
    () => canPerformAction(user, requiredPermissions, requireAll),
    [user, requiredPermissions, requireAll],
  )
}

/**
 * Hook for getting missing permissions for an action
 */
export function useMissingPermissions(
  requiredPermissions: PermissionKey[],
): PermissionKey[] {
  const user = useAuthStore(state => state.user)
  return useMemo(
    () => getMissingPermissions(user, requiredPermissions),
    [user, requiredPermissions],
  )
}

/**
 * Hook for permission utilities
 */
export function usePermissionUtils() {
  return {
    expandWildcardPermission,
    isWildcardPermission,
    getPermissionDisplayName,
    getPermissionDescription,
    groupPermissionsByCategory,
    getAllPermissionsGrouped,
    formatPermissionsForDisplay,
  }
}

/**
 * Hook for admin-specific permission checks
 */
export function useAdminPermissions() {
  const { hasPermission, hasAnyPermission } = usePermissions()

  return {
    canViewUsers: hasPermission('users::read'),
    canEditUsers: hasPermission('users::edit'),
    canCreateUsers: hasPermission('users::create'),
    canDeleteUsers: hasPermission('users::delete'),
    canManageUsers: hasAnyPermission([
      'users::edit',
      'users::create',
      'users::delete',
    ]),

    canViewGroups: hasPermission('groups::read'),
    canEditGroups: hasPermission('groups::edit'),
    canCreateGroups: hasPermission('groups::create'),
    canDeleteGroups: hasPermission('groups::delete'),
    canManageGroups: hasAnyPermission([
      'groups::edit',
      'groups::create',
      'groups::delete',
    ]),

    canViewConfig: hasPermission('config::user-registration::read'),
    canEditConfig: hasPermission('config::user-registration::edit'),

    isAdmin: hasPermission('*'),
  }
}

/**
 * Hook for user-specific permission checks
 */
export function useUserPermissions() {
  const { hasPermission } = usePermissions()

  return {
    canUseChat: hasPermission('chat::use'),
    canEditProfile: hasPermission('profile::edit'),
  }
}

/**
 * Hook for conditional rendering based on permissions
 */
export function usePermissionGuard() {
  const { hasPermission, hasAnyPermission, hasAllPermissions } =
    usePermissions()

  return {
    /**
     * Render children only if user has the required permission
     */
    requirePermission: (
      permission: PermissionKey,
      children: React.ReactNode,
    ) => (hasPermission(permission) ? children : null),

    /**
     * Render children only if user has any of the required permissions
     */
    requireAnyPermission: (
      permissions: PermissionKey[],
      children: React.ReactNode,
    ) => (hasAnyPermission(permissions) ? children : null),

    /**
     * Render children only if user has all of the required permissions
     */
    requireAllPermissions: (
      permissions: PermissionKey[],
      children: React.ReactNode,
    ) => (hasAllPermissions(permissions) ? children : null),
  }
}
