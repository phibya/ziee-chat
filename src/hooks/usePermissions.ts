import { useAuthStore } from '../store/auth'
import {
  hasAllPermissions,
  hasAnyPermission,
  hasPermission,
  getUserEffectivePermissions,
  expandWildcardPermission,
  isWildcardPermission,
  getPermissionDisplayName,
  groupPermissionsByCategory,
} from '../utils/permissions'
import type { PermissionKey } from '../api/enpoints'

/**
 * React hook for checking user permissions with AWS-style permission support
 */
export function usePermissions() {
  const user = useAuthStore(state => state.user)

  return {
    /**
     * Check if the current user has a specific permission
     */
    hasPermission: (permission: PermissionKey) =>
      hasPermission(user, permission),

    /**
     * Check if the current user has any of the specified permissions
     */
    hasAnyPermission: (permissions: PermissionKey[]) =>
      hasAnyPermission(user, permissions),

    /**
     * Check if the current user has all of the specified permissions
     */
    hasAllPermissions: (permissions: PermissionKey[]) =>
      hasAllPermissions(user, permissions),

    /**
     * Get all effective permissions for the current user (including expanded wildcards)
     */
    getEffectivePermissions: () =>
      getUserEffectivePermissions(user),

    /**
     * Expand a wildcard permission to all permissions it grants
     */
    expandWildcardPermission: (wildcard: PermissionKey) =>
      expandWildcardPermission(wildcard),

    /**
     * Check if a permission is a wildcard permission
     */
    isWildcardPermission: (permission: PermissionKey) =>
      isWildcardPermission(permission),

    /**
     * Get the display name for a permission
     */
    getPermissionDisplayName: (permission: PermissionKey) =>
      getPermissionDisplayName(permission),

    /**
     * Group permissions by category for display
     */
    groupPermissionsByCategory: (permissions: PermissionKey[]) =>
      groupPermissionsByCategory(permissions),

    /**
     * Get the current user object
     */
    user,
  }
}
