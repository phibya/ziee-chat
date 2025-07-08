/**
 * Centralized permissions module
 * 
 * This module provides a single entry point for all permission-related functionality:
 * - Permission constants and types
 * - Permission utility functions
 * - Permission-related React hooks
 * - Permission checking and validation
 * 
 * Usage:
 * ```typescript
 * import { PermissionKeys, usePermissions, hasPermission } from '@/permissions'
 * 
 * // In components
 * const { hasPermission } = usePermissions()
 * const canEdit = hasPermission(PermissionKeys.USERS_EDIT)
 * 
 * // In utilities
 * const userCanEdit = hasPermission(user, PermissionKeys.USERS_EDIT)
 * ```
 */

// Constants and types
export {
  PermissionKeys,
  PermissionCategories,
  PermissionDisplayNames,
  PermissionDescriptions,
  WildcardPermissions,
  type PermissionKey,
} from './constants'

export type {
  User,
  UserEmail,
  UserServices,
  UserGroup,
  CreateUserGroupRequest,
  UpdateUserGroupRequest,
  UpdateUserRequest,
  ResetPasswordRequest,
  AssignUserToGroupRequest,
  UserListResponse,
  UserGroupListResponse,
  PermissionCheckResult,
  PermissionContext,
} from './types'

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