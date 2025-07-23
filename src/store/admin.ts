import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../api/client'
import { Assistant } from '../types/api/assistant'
import { CreateUserGroupRequest, User, UserGroup } from '../types/api/user'
import { UpdateProxySettingsRequest } from '../types/api/config'

// Using API types now - User and UserGroup imported above

interface GroupMember {
  id: string
  username: string
  email: string
  is_active: boolean
  joined_at: string
}

type ProxySettings = UpdateProxySettingsRequest

interface AdminState {
  // Users
  users: User[]
  loadingUsers: boolean

  // User Groups
  groups: UserGroup[]
  loadingGroups: boolean
  currentGroupMembers: GroupMember[]
  loadingGroupMembers: boolean

  // Assistants
  assistants: Assistant[]
  loading: boolean

  // Settings
  userRegistrationEnabled: boolean
  loadingRegistrationSettings: boolean

  // Proxy settings
  proxySettings: ProxySettings | null
  loadingProxySettings: boolean

  // Global states
  creating: boolean
  updating: boolean
  deleting: boolean
  error: string | null
}

export const useAdminStore = create<AdminState>()(
  subscribeWithSelector(
    (): AdminState => ({
      // Initial state
      users: [],
      loadingUsers: false,
      groups: [],
      loadingGroups: false,
      currentGroupMembers: [],
      loadingGroupMembers: false,
      assistants: [],
      loading: false,
      userRegistrationEnabled: true,
      loadingRegistrationSettings: false,
      proxySettings: null,
      loadingProxySettings: false,
      creating: false,
      updating: false,
      deleting: false,
      error: null,
    }),
  ),
)

// User actions
export const loadAllSystemUsers = async (): Promise<void> => {
  try {
    useAdminStore.setState({ loadingUsers: true, error: null })

    const response = await ApiClient.Admin.listUsers({
      page: 1,
      per_page: 50,
    })

    useAdminStore.setState({
      users: response.users,
      loadingUsers: false,
    })
  } catch (error) {
    useAdminStore.setState({
      error: error instanceof Error ? error.message : 'Failed to load users',
      loadingUsers: false,
    })
    throw error
  }
}

export const updateSystemUser = async (
  id: string,
  data: Partial<User>,
): Promise<User> => {
  try {
    useAdminStore.setState({ updating: true, error: null })

    const user = await ApiClient.Admin.updateUser({ user_id: id, ...data })

    useAdminStore.setState(state => ({
      users: state.users.map(u => (u.id === id ? user : u)),
      updating: false,
    }))

    return user
  } catch (error) {
    useAdminStore.setState({
      error: error instanceof Error ? error.message : 'Failed to update user',
      updating: false,
    })
    throw error
  }
}

export const resetSystemUserPassword = async (
  id: string,
  newPassword: string,
): Promise<void> => {
  try {
    useAdminStore.setState({ updating: true, error: null })

    await ApiClient.Admin.resetPassword({
      user_id: id,
      new_password: newPassword,
    })

    useAdminStore.setState({ updating: false })
  } catch (error) {
    useAdminStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to reset password',
      updating: false,
    })
    throw error
  }
}

export const toggleSystemUserActiveStatus = async (id: string): Promise<void> => {
  try {
    useAdminStore.setState({ updating: true, error: null })

    await ApiClient.Admin.toggleUserActive({ user_id: id })

    useAdminStore.setState(state => ({
      users: state.users.map(u =>
        u.id === id ? { ...u, is_active: !u.is_active } : u,
      ),
      updating: false,
    }))
  } catch (error) {
    useAdminStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to toggle user status',
      updating: false,
    })
    throw error
  }
}

// User Group actions
export const loadAllUserGroups = async (): Promise<void> => {
  try {
    useAdminStore.setState({ loadingGroups: true, error: null })

    const response = await ApiClient.Admin.listGroups({
      page: 1,
      per_page: 50,
    })

    useAdminStore.setState({
      groups: response.groups,
      loadingGroups: false,
    })
  } catch (error) {
    useAdminStore.setState({
      error: error instanceof Error ? error.message : 'Failed to load groups',
      loadingGroups: false,
    })
    throw error
  }
}

export const createNewUserGroup = async (
  data: CreateUserGroupRequest,
): Promise<UserGroup> => {
  try {
    useAdminStore.setState({ creating: true, error: null })

    const group = await ApiClient.Admin.createGroup(data)

    useAdminStore.setState(state => ({
      groups: [...state.groups, group],
      creating: false,
    }))

    return group
  } catch (error) {
    useAdminStore.setState({
      error: error instanceof Error ? error.message : 'Failed to create group',
      creating: false,
    })
    throw error
  }
}

export const updateUserGroup = async (
  id: string,
  data: Partial<UserGroup>,
): Promise<UserGroup> => {
  try {
    useAdminStore.setState({ updating: true, error: null })

    const group = await ApiClient.Admin.updateGroup({
      group_id: id,
      ...data,
    })

    useAdminStore.setState(state => ({
      groups: state.groups.map(g => (g.id === id ? group : g)),
      updating: false,
    }))

    return group
  } catch (error) {
    useAdminStore.setState({
      error: error instanceof Error ? error.message : 'Failed to update group',
      updating: false,
    })
    throw error
  }
}

export const deleteUserGroup = async (id: string): Promise<void> => {
  try {
    useAdminStore.setState({ deleting: true, error: null })

    await ApiClient.Admin.deleteGroup({ group_id: id })

    useAdminStore.setState(state => ({
      groups: state.groups.filter(g => g.id !== id),
      deleting: false,
    }))
  } catch (error) {
    useAdminStore.setState({
      error: error instanceof Error ? error.message : 'Failed to delete group',
      deleting: false,
    })
    throw error
  }
}

export const loadUserGroupMembers = async (groupId: string): Promise<void> => {
  try {
    useAdminStore.setState({ loadingGroupMembers: true, error: null })

    const response = await ApiClient.Admin.getGroupMembers({
      group_id: groupId,
      page: 1,
      per_page: 50,
    })

    useAdminStore.setState({
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
    useAdminStore.setState({
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
  try {
    useAdminStore.setState({ updating: true, error: null })

    await ApiClient.Admin.assignUserToGroup({
      user_id: userId,
      group_id: groupId,
    })

    // Reload group members if we're viewing this group
    const { currentGroupMembers } = useAdminStore.getState()
    if (currentGroupMembers.length > 0) {
      await loadUserGroupMembers(groupId)
    }

    useAdminStore.setState({ updating: false })
  } catch (error) {
    useAdminStore.setState({
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
  try {
    useAdminStore.setState({ updating: true, error: null })

    await ApiClient.Admin.removeUserFromGroup({
      user_id: userId,
      group_id: groupId,
    })

    // Remove from current group members
    useAdminStore.setState(state => ({
      currentGroupMembers: state.currentGroupMembers.filter(
        m => m.id !== userId,
      ),
      updating: false,
    }))
  } catch (error) {
    useAdminStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to remove user from group',
      updating: false,
    })
    throw error
  }
}

// Registration settings
export const loadSystemUserRegistrationSettings = async (): Promise<void> => {
  try {
    useAdminStore.setState({ loadingRegistrationSettings: true, error: null })

    const { enabled } = await ApiClient.Admin.getUserRegistrationStatus()

    useAdminStore.setState({
      userRegistrationEnabled: enabled,
      loadingRegistrationSettings: false,
    })
  } catch (error) {
    useAdminStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to load registration settings',
      loadingRegistrationSettings: false,
    })
    throw error
  }
}

export const updateSystemUserRegistrationSettings = async (
  enabled: boolean,
): Promise<void> => {
  try {
    useAdminStore.setState({ updating: true, error: null })

    await ApiClient.Admin.updateUserRegistrationStatus({ enabled })

    useAdminStore.setState({
      userRegistrationEnabled: enabled,
      updating: false,
    })
  } catch (error) {
    useAdminStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to update registration settings',
      updating: false,
    })
    throw error
  }
}

// Assistant actions for admin
export const loadSystemAdminAssistants = async (): Promise<void> => {
  try {
    useAdminStore.setState({ loading: true, error: null })
    const response = await ApiClient.Assistant.list({
      page: 1,
      per_page: 50,
    })
    useAdminStore.setState({
      assistants: response.assistants,
      loading: false,
    })
  } catch (error) {
    useAdminStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to load assistants',
      loading: false,
    })
    throw error
  }
}

export const createSystemAdminAssistant = async (
  data: Partial<Assistant>,
): Promise<Assistant> => {
  try {
    useAdminStore.setState({ creating: true, error: null })
    const assistant = await ApiClient.Assistant.create(data as any)
    useAdminStore.setState(state => ({
      assistants: [...state.assistants, assistant],
      creating: false,
    }))
    return assistant
  } catch (error) {
    useAdminStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to create assistant',
      creating: false,
    })
    throw error
  }
}

export const updateSystemAdminAssistant = async (
  id: string,
  data: Partial<Assistant>,
): Promise<Assistant> => {
  try {
    useAdminStore.setState({ updating: true, error: null })
    const assistant = await ApiClient.Assistant.update({
      assistant_id: id,
      ...data,
    })
    useAdminStore.setState(state => ({
      assistants: state.assistants.map(a => (a.id === id ? assistant : a)),
      updating: false,
    }))
    return assistant
  } catch (error) {
    useAdminStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to update assistant',
      updating: false,
    })
    throw error
  }
}

export const deleteSystemAdminAssistant = async (id: string): Promise<void> => {
  try {
    useAdminStore.setState({ deleting: true, error: null })
    await ApiClient.Assistant.delete({ assistant_id: id })
    useAdminStore.setState(state => ({
      assistants: state.assistants.filter(a => a.id !== id),
      deleting: false,
    }))
  } catch (error) {
    useAdminStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to delete assistant',
      deleting: false,
    })
    throw error
  }
}

// Proxy settings
export const loadSystemProxySettings = async (): Promise<void> => {
  try {
    useAdminStore.setState({ loadingProxySettings: true, error: null })

    const settings = await ApiClient.Admin.getProxySettings()

    useAdminStore.setState({
      proxySettings: {
        enabled: settings.enabled,
        url: settings.url,
        username: settings.username,
        password: settings.password,
        no_proxy: settings.no_proxy,
        ignore_ssl_certificates: settings.ignore_ssl_certificates,
        proxy_ssl: settings.proxy_ssl,
        proxy_host_ssl: settings.proxy_host_ssl,
        peer_ssl: settings.peer_ssl,
        host_ssl: settings.host_ssl,
      },
      loadingProxySettings: false,
    })
  } catch (error) {
    useAdminStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to load proxy settings',
      loadingProxySettings: false,
    })
    throw error
  }
}

export const updateSystemProxySettings = async (
  settings: ProxySettings,
): Promise<void> => {
  try {
    useAdminStore.setState({ updating: true, error: null })

    await ApiClient.Admin.updateProxySettings(settings)

    useAdminStore.setState({
      proxySettings: settings,
      updating: false,
    })
  } catch (error) {
    useAdminStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to update proxy settings',
      updating: false,
    })
    throw error
  }
}

export const updateSystemDefaultLanguage = async (
  language: string,
): Promise<void> => {
  try {
    useAdminStore.setState({ updating: true, error: null })

    await ApiClient.Admin.updateDefaultLanguage({ language })

    useAdminStore.setState({ updating: false })
  } catch (error) {
    useAdminStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to update default language',
      updating: false,
    })
    throw error
  }
}

export const clearSystemAdminError = (): void => {
  useAdminStore.setState({ error: null })
}
