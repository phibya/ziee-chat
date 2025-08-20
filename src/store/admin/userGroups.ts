import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../../api/client'
import { CreateUserGroupRequest, UserGroup } from '../../types'

interface GroupMember {
  id: string
  username: string
  email: string
  is_active: boolean
  joined_at: string
}

interface AdminUserGroupsState {
  // Data
  groups: UserGroup[]
  currentGroupMembers: GroupMember[]
  total: number
  currentPage: number
  pageSize: number
  isInitialized: boolean
  currentGroupId: string | null

  // Loading states
  loading: boolean
  loadingGroups: boolean
  loadingGroupMembers: boolean
  creating: boolean
  updating: boolean
  deleting: boolean

  // Error state
  error: string | null
}

export const useAdminUserGroupsStore = create<AdminUserGroupsState>()(
  subscribeWithSelector(
    (): AdminUserGroupsState => ({
      // Initial state
      groups: [],
      currentGroupMembers: [],
      total: 0,
      currentPage: 1,
      pageSize: 10,
      isInitialized: false,
      currentGroupId: null,
      loading: false,
      loadingGroups: false,
      loadingGroupMembers: false,
      creating: false,
      updating: false,
      deleting: false,
      error: null,
    }),
  ),
)

// User Group actions
export const loadUserGroups = async (
  page?: number,
  pageSize?: number,
): Promise<void> => {
  try {
    const currentState = useAdminUserGroupsStore.getState()
    const requestPage = page || currentState.currentPage
    const requestPageSize = pageSize || currentState.pageSize

    // Skip if already initialized and loading first page without explicit page parameter
    if (currentState.isInitialized && currentState.loadingGroups && !page) {
      return
    }

    useAdminUserGroupsStore.setState({ loadingGroups: true, error: null })

    const response = await ApiClient.Admin.listGroups({
      page: requestPage,
      per_page: requestPageSize,
    })

    useAdminUserGroupsStore.setState({
      groups: response.groups,
      total: response.total,
      currentPage: response.page,
      pageSize: response.per_page,
      isInitialized: true,
      loadingGroups: false,
    })
  } catch (error) {
    useAdminUserGroupsStore.setState({
      error: error instanceof Error ? error.message : 'Failed to load groups',
      loadingGroups: false,
    })
    throw error
  }
}

export const createNewUserGroup = async (
  data: CreateUserGroupRequest,
): Promise<UserGroup | undefined> => {
  const state = useAdminUserGroupsStore.getState()
  if (state.creating) {
    return
  }

  try {
    useAdminUserGroupsStore.setState({ creating: true, error: null })

    const group = await ApiClient.Admin.createGroup(data)

    useAdminUserGroupsStore.setState(state => ({
      groups: [...state.groups, group],
      creating: false,
    }))

    return group
  } catch (error) {
    useAdminUserGroupsStore.setState({
      error: error instanceof Error ? error.message : 'Failed to create group',
      creating: false,
    })
    throw error
  }
}

export const updateUserGroup = async (
  id: string,
  data: Partial<UserGroup>,
): Promise<UserGroup | undefined> => {
  const state = useAdminUserGroupsStore.getState()
  if (state.updating) {
    return
  }

  try {
    useAdminUserGroupsStore.setState({ updating: true, error: null })

    const group = await ApiClient.Admin.updateGroup({
      group_id: id,
      ...data,
    })

    useAdminUserGroupsStore.setState(state => ({
      groups: state.groups.map(g => (g.id === id ? group : g)),
      updating: false,
    }))

    return group
  } catch (error) {
    useAdminUserGroupsStore.setState({
      error: error instanceof Error ? error.message : 'Failed to update group',
      updating: false,
    })
    throw error
  }
}

export const deleteUserGroup = async (id: string): Promise<void> => {
  const state = useAdminUserGroupsStore.getState()
  if (state.deleting) {
    return
  }

  try {
    useAdminUserGroupsStore.setState({ deleting: true, error: null })

    await ApiClient.Admin.deleteGroup({ group_id: id })

    useAdminUserGroupsStore.setState(state => ({
      groups: state.groups.filter(g => g.id !== id),
      deleting: false,
    }))
  } catch (error) {
    useAdminUserGroupsStore.setState({
      error: error instanceof Error ? error.message : 'Failed to delete group',
      deleting: false,
    })
    throw error
  }
}

export const loadUserGroupMembers = async (groupId: string): Promise<void> => {
  try {
    const currentState = useAdminUserGroupsStore.getState()

    // Skip if already loading members for the same group
    if (
      currentState.loadingGroupMembers &&
      currentState.currentGroupId === groupId
    ) {
      return
    }

    useAdminUserGroupsStore.setState({
      loadingGroupMembers: true,
      error: null,
      currentGroupId: groupId,
    })

    const response = await ApiClient.Admin.getGroupMembers({
      group_id: groupId,
      page: 1,
      per_page: 50,
    })

    useAdminUserGroupsStore.setState({
      currentGroupMembers: response.users.map(u => ({
        id: u.id,
        username: u.username,
        email: u.emails?.[0]?.address || '',
        is_active: u.is_active,
        joined_at: new Date().toISOString(),
      })),
      loadingGroupMembers: false,
    })
  } catch (error) {
    useAdminUserGroupsStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to load group members',
      loadingGroupMembers: false,
    })
    throw error
  }
}

export const assignUserToUserGroup = async (
  userId: string,
  groupId: string,
): Promise<void> => {
  const state = useAdminUserGroupsStore.getState()
  if (state.updating) {
    return
  }

  try {
    useAdminUserGroupsStore.setState({ updating: true, error: null })

    await ApiClient.Admin.assignUserToGroup({
      user_id: userId,
      group_id: groupId,
    })

    // Reload group members if we're viewing this group
    const { currentGroupMembers } = useAdminUserGroupsStore.getState()
    if (currentGroupMembers.length > 0) {
      await loadUserGroupMembers(groupId)
    }

    useAdminUserGroupsStore.setState({ updating: false })
  } catch (error) {
    useAdminUserGroupsStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to assign user to group',
      updating: false,
    })
    throw error
  }
}

export const removeUserFromUserGroup = async (
  userId: string,
  groupId: string,
): Promise<void> => {
  const state = useAdminUserGroupsStore.getState()
  if (state.updating) {
    return
  }

  try {
    useAdminUserGroupsStore.setState({ updating: true, error: null })

    await ApiClient.Admin.removeUserFromGroup({
      user_id: userId,
      group_id: groupId,
    })

    // Remove from current group members
    useAdminUserGroupsStore.setState(state => ({
      currentGroupMembers: state.currentGroupMembers.filter(
        m => m.id !== userId,
      ),
      updating: false,
    }))
  } catch (error) {
    useAdminUserGroupsStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to remove user from group',
      updating: false,
    })
    throw error
  }
}

export const clearAdminUserGroupsStoreError = (): void => {
  useAdminUserGroupsStore.setState({ error: null })
}
