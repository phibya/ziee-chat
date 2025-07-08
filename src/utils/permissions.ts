/**
 * Legacy permission utilities - re-export from centralized permissions module
 * @deprecated Use functions from '../permissions' instead
 */
export {
  hasPermission,
  hasAnyPermission,
  hasAllPermissions,
  getUserEffectivePermissions,
  expandWildcardPermission,
  isWildcardPermission,
  getPermissionDisplayName,
  groupPermissionsByCategory,
  getPermissionCategory,
} from '../permissions'

// Re-export types for backward compatibility
export type { PermissionKey, User, UserGroup } from '../permissions'
