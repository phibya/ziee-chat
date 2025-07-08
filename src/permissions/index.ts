// Constants and types
export { PermissionKeys, type PermissionKey } from './constants'
export { PermissionDescription } from './constants'

// Utility functions
export {
  hasPermission,
  hasAnyPermission,
  hasAllPermissions,
  getUserEffectivePermissions,
  expandWildcardPermission,
  isWildcardPermission,
  getPermissionDisplayName,
  getPermissionDescription,
  getPermissionCategory,
  groupPermissionsByCategory,
  getAllPermissionsGrouped,
  isValidPermission,
  getPermissionsForCategory,
  canPerformAction,
  getMissingPermissions,
  formatPermissionsForDisplay,
} from './utils'

// React hooks
export {
  usePermissions,
  useHasPermission,
  useHasAnyPermission,
  useHasAllPermissions,
  useEffectivePermissions,
  usePermissionsByCategory,
  useCanPerformAction,
  useMissingPermissions,
  usePermissionUtils,
  useAdminPermissions,
  useUserPermissions,
  usePermissionGuard,
} from './hooks'

// Re-export for convenience
export * from './constants'
export * from './types'
export * from './utils'
export * from './hooks'
