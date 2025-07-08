/**
 * AWS-style permission constants for fine-grained access control
 * This file contains all permission constants used throughout the application
 */

// AWS-style permissions matching server-side implementation
export const PermissionKeys = {
  // User management permissions
  USERS_READ: 'users::read',
  USERS_EDIT: 'users::edit',
  USERS_DELETE: 'users::delete',
  USERS_CREATE: 'users::create',
  USERS_ALL: 'users::*',
  
  // Group management permissions
  GROUPS_READ: 'groups::read',
  GROUPS_EDIT: 'groups::edit',
  GROUPS_DELETE: 'groups::delete',
  GROUPS_CREATE: 'groups::create',
  GROUPS_ALL: 'groups::*',
  
  // Fine-grained configuration permissions
  CONFIG_USER_REGISTRATION_READ: 'config::user-registration::read',
  CONFIG_USER_REGISTRATION_EDIT: 'config::user-registration::edit',
  
  // Chat permissions
  CHAT_USE: 'chat::use',
  
  // Profile permissions
  PROFILE_EDIT: 'profile::edit',
  
  // Wildcard permissions
  ALL: '*',
} as const

export type PermissionKey = typeof PermissionKeys[keyof typeof PermissionKeys]

/**
 * Permission categories for grouping and organization
 */
export const PermissionCategories = {
  USERS: 'users',
  GROUPS: 'groups',
  CONFIG: 'config',
  CHAT: 'chat',
  PROFILE: 'profile',
} as const

/**
 * Permission display names for UI
 */
export const PermissionDisplayNames: Record<PermissionKey, string> = {
  [PermissionKeys.USERS_READ]: 'View Users',
  [PermissionKeys.USERS_EDIT]: 'Edit Users',
  [PermissionKeys.USERS_DELETE]: 'Delete Users',
  [PermissionKeys.USERS_CREATE]: 'Create Users',
  [PermissionKeys.USERS_ALL]: 'All User Permissions',
  [PermissionKeys.GROUPS_READ]: 'View Groups',
  [PermissionKeys.GROUPS_EDIT]: 'Edit Groups',
  [PermissionKeys.GROUPS_DELETE]: 'Delete Groups',
  [PermissionKeys.GROUPS_CREATE]: 'Create Groups',
  [PermissionKeys.GROUPS_ALL]: 'All Group Permissions',
  [PermissionKeys.CONFIG_USER_REGISTRATION_READ]: 'View Registration Settings',
  [PermissionKeys.CONFIG_USER_REGISTRATION_EDIT]: 'Edit Registration Settings',
  [PermissionKeys.CHAT_USE]: 'Use Chat',
  [PermissionKeys.PROFILE_EDIT]: 'Edit Profile',
  [PermissionKeys.ALL]: 'All Permissions',
}

/**
 * Permission descriptions for tooltips and help text
 */
export const PermissionDescriptions: Record<PermissionKey, string> = {
  [PermissionKeys.USERS_READ]: 'Allows viewing user lists and user details',
  [PermissionKeys.USERS_EDIT]: 'Allows editing user information, resetting passwords, and managing user status',
  [PermissionKeys.USERS_DELETE]: 'Allows deleting user accounts',
  [PermissionKeys.USERS_CREATE]: 'Allows creating new user accounts',
  [PermissionKeys.USERS_ALL]: 'Grants all user management permissions',
  [PermissionKeys.GROUPS_READ]: 'Allows viewing group lists and group details',
  [PermissionKeys.GROUPS_EDIT]: 'Allows editing group information and managing group memberships',
  [PermissionKeys.GROUPS_DELETE]: 'Allows deleting user groups',
  [PermissionKeys.GROUPS_CREATE]: 'Allows creating new user groups',
  [PermissionKeys.GROUPS_ALL]: 'Grants all group management permissions',
  [PermissionKeys.CONFIG_USER_REGISTRATION_READ]: 'Allows viewing user registration settings',
  [PermissionKeys.CONFIG_USER_REGISTRATION_EDIT]: 'Allows modifying user registration settings',
  [PermissionKeys.CHAT_USE]: 'Allows using chat functionality',
  [PermissionKeys.PROFILE_EDIT]: 'Allows editing own profile information',
  [PermissionKeys.ALL]: 'Grants all permissions in the system',
}

/**
 * Wildcard permission mappings
 */
export const WildcardPermissions: Record<string, PermissionKey[]> = {
  [PermissionKeys.USERS_ALL]: [
    PermissionKeys.USERS_READ,
    PermissionKeys.USERS_EDIT,
    PermissionKeys.USERS_DELETE,
    PermissionKeys.USERS_CREATE,
  ],
  [PermissionKeys.GROUPS_ALL]: [
    PermissionKeys.GROUPS_READ,
    PermissionKeys.GROUPS_EDIT,
    PermissionKeys.GROUPS_DELETE,
    PermissionKeys.GROUPS_CREATE,
  ],
  [PermissionKeys.ALL]: [
    PermissionKeys.USERS_READ,
    PermissionKeys.USERS_EDIT,
    PermissionKeys.USERS_DELETE,
    PermissionKeys.USERS_CREATE,
    PermissionKeys.GROUPS_READ,
    PermissionKeys.GROUPS_EDIT,
    PermissionKeys.GROUPS_DELETE,
    PermissionKeys.GROUPS_CREATE,
    PermissionKeys.CONFIG_USER_REGISTRATION_READ,
    PermissionKeys.CONFIG_USER_REGISTRATION_EDIT,
    PermissionKeys.CHAT_USE,
    PermissionKeys.PROFILE_EDIT,
  ],
}