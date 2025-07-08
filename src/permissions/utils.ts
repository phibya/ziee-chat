/**
 * Permission utility functions
 */

import { User, UserGroup } from '../types'
import { Permission, PermissionDescription, PermissionKey, PermissionKeys } from './constants.ts'

/**
 * Check if a user has a specific permission based on their group memberships
 * Supports wildcard permissions (e.g., "users::*" grants all users permissions)
 */
export function hasPermission(
  user: User | null,
  permission: PermissionKey,
): boolean {
  if (!user || !user.groups) {
    return false
  }

  return user.groups.some((group: UserGroup) => {
    if (!group.is_active || !group.permissions) {
      return false
    }

    // Check for exact permission match
    if (group.permissions.includes(permission)) {
      return true
    }

    // Check for global wildcard
    if (group.permissions.includes(Permission.all)) {
      return true
    }

    // Check for category wildcards (e.g., "users::*" for "users::read")
    const category = getPermissionCategory(permission)
    if (category) {
      const wildcard = `${category}::*` as PermissionKey
      if (group.permissions.includes(wildcard)) {
        return true
      }
    }

    return false
  })
}

/**
 * Extract the category from a permission string (e.g., "users::read" -> "users")
 */
export function getPermissionCategory(
  permission: PermissionKey,
): string | null {
  return permission.replace(/::[^:]*$/, '') || null
}

/**
 * Check if a user has any of the specified permissions
 */
export function hasAnyPermission(
  user: User | null,
  permissions: PermissionKey[],
): boolean {
  return permissions.some(permission => hasPermission(user, permission))
}

/**
 * Check if a user has all of the specified permissions
 */
export function hasAllPermissions(
  user: User | null,
  permissions: PermissionKey[],
): boolean {
  return permissions.every(permission => hasPermission(user, permission))
}

/**
 * Get all permissions that a wildcard permission grants
 */
export function expandWildcardPermission(
  wildcard: PermissionKey,
): PermissionKey[] {
  if (!isWildcardPermission(wildcard)) {
    return []
  }
  // Handle global wildcard
  if (wildcard === Permission.all) {
    return PermissionKeys
  }
  // Handle category wildcards (e.g., "users::*")
  const category = getPermissionCategory(wildcard)
  if (!category) {
    return []
  }

  return PermissionKeys.filter(permission =>
    permission.startsWith(category + '::'),
  )
}

/**
 * Get all effective permissions for a user (including expanded wildcards)
 */
export function getUserEffectivePermissions(
  user: User | null,
): PermissionKey[] {
  if (!user || !user.groups) {
    return []
  }

  const effectivePermissions = new Set<PermissionKey>()

  user.groups.forEach((group: UserGroup) => {
    if (!group.is_active || !group.permissions) {
      return
    }

    group.permissions.forEach(permission => {
      // Add the permission itself
      effectivePermissions.add(permission)

      // If it's a wildcard, add all permissions it grants
      const expandedPermissions = expandWildcardPermission(permission)
      expandedPermissions.forEach(expanded => {
        effectivePermissions.add(expanded)
      })
    })
  })

  return Array.from(effectivePermissions)
}

/**
 * Check if a permission is a wildcard permission
 */
export function isWildcardPermission(permission: PermissionKey): boolean {
  return permission === Permission.all || permission.endsWith('::*')
}

/**
 * Get the display name for a permission
 */
export function getPermissionDisplayName(permission: PermissionKey): string {
  return PermissionDescription[permission] || permission
}

/**
 * Get the description for a permission
 */
export function getPermissionDescription(permission: PermissionKey): string {
  return PermissionDescription[permission] || ''
}

/**
 * Group permissions by category for display
 */
export function groupPermissionsByCategory(
  permissions: PermissionKey[],
): Record<string, PermissionKey[]> {
  const grouped: Record<string, PermissionKey[]> = {}

  permissions.forEach(permission => {
    const category = getPermissionCategory(permission) || 'Other'
    if (!grouped[category]) {
      grouped[category] = []
    }
    grouped[category].push(permission)
  })

  return grouped
}

/**
 * Get all available permissions grouped by category
 */
export function getAllPermissionsGrouped(): Record<string, PermissionKey[]> {
  const allPermissions = Object.values(PermissionKeys)
  return groupPermissionsByCategory(allPermissions)
}

/**
 * Validate if a permission string is valid
 */
export function isValidPermission(
  permission: string,
): permission is PermissionKey {
  return PermissionKeys.includes(permission as PermissionKey)
}

/**
 * Get permissions for a specific category
 */
export function getPermissionsForCategory(category: string): PermissionKey[] {
  return Object.values(PermissionKeys).filter(
    permission => getPermissionCategory(permission) === category,
  )
}

/**
 * Check if a user can perform a specific action based on required permissions
 */
export function canPerformAction(
  user: User | null,
  requiredPermissions: PermissionKey[],
  requireAll: boolean = false,
): boolean {
  if (requireAll) {
    return hasAllPermissions(user, requiredPermissions)
  }
  return hasAnyPermission(user, requiredPermissions)
}

/**
 * Get missing permissions for a user to perform an action
 */
export function getMissingPermissions(
  user: User | null,
  requiredPermissions: PermissionKey[],
): PermissionKey[] {
  return requiredPermissions.filter(
    permission => !hasPermission(user, permission),
  )
}

/**
 * Format permissions for display in lists
 */
export function formatPermissionsForDisplay(
  permissions: PermissionKey[],
): Array<{
  key: PermissionKey
  name: string
  description: string
  category: string
}> {
  return permissions.map(permission => ({
    key: permission,
    name: getPermissionDisplayName(permission),
    description: getPermissionDescription(permission),
    category: getPermissionCategory(permission) || 'Other',
  }))
}
