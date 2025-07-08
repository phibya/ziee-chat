import { useAuthStore } from '../store/auth'
import {
  hasAllPermissions,
  hasAnyPermission,
  hasPermission,
  PermissionKey,
} from '../utils/permissions'

/**
 * React hook for checking user permissions
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
     * Get the current user object
     */
    user,
  }
}
