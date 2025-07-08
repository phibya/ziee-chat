import { UserGroup } from './user.ts'

/**
 * User interface for permission checking
 */

export interface UserGroupListResponse {
  groups: UserGroup[]
  total: number
  page: number
  per_page: number
}
