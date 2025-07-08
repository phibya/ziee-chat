import type { Permissions, User, UserGroup } from '../api/enpoints'

export type PermissionKey = keyof Permissions

/**
 * Check if a user has a specific permission based on their group memberships
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

    // Check if the group has the specific permission
    return group.permissions[permission]
  })
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
