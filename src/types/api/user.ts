import type { PermissionKey } from '../../permissions'

export interface User {
  id: string
  username: string
  emails: UserEmail[]
  created_at: string
  profile?: any
  services: UserServices
  is_active: boolean
  last_login_at?: string
  updated_at: string
  groups: UserGroup[]
}

/**
 * User email interface
 */
export interface UserEmail {
  address: string
  verified: boolean
}

/**
 * User services interface
 */
export interface UserServices {
  facebook?: any
  resume?: any
  password?: any
}

/**
 * User group interface with array-based permissions
 */
export interface UserGroup {
  id: string
  name: string
  description?: string
  permissions: PermissionKey[] // Array of permission strings
  is_active: boolean
  created_at: string
  updated_at: string
}

/**
 * Request interfaces for group management
 */
export interface CreateUserGroupRequest {
  name: string
  description?: string
  permissions: PermissionKey[]
}

export interface UpdateUserGroupRequest {
  group_id: string
  name?: string
  description?: string
  permissions?: PermissionKey[]
  is_active?: boolean
}

/**
 * Request interfaces for user management
 */
export interface UpdateUserRequest {
  user_id: string
  username?: string
  email?: string
  is_active?: boolean
  profile?: any
}

export interface ResetPasswordRequest {
  user_id: string
  new_password: string
}

export interface AssignUserToGroupRequest {
  user_id: string
  group_id: string
}

/**
 * Response interfaces
 */
export interface UserListResponse {
  users: User[]
  total: number
  page: number
  per_page: number
}

export interface CreateUserRequest {
  username: string
  email: string
  password: string
  profile?: any
}
