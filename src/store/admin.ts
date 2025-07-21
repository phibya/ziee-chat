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

  // User actions
  loadUsers: () => Promise<void>
  updateUser: (id: string, data: Partial<User>) => Promise<User>
  resetUserPassword: (id: string, newPassword: string) => Promise<void>
  toggleUserActive: (id: string) => Promise<void>

  // User Group actions
  loadGroups: () => Promise<void>
  createGroup: (data: CreateUserGroupRequest) => Promise<UserGroup>
  updateGroup: (id: string, data: Partial<UserGroup>) => Promise<UserGroup>
  deleteGroup: (id: string) => Promise<void>
  loadGroupMembers: (groupId: string) => Promise<void>
  assignUserToGroup: (userId: string, groupId: string) => Promise<void>
  removeUserFromGroup: (userId: string, groupId: string) => Promise<void>

  // Registration settings
  loadUserRegistrationSettings: () => Promise<void>
  updateUserRegistrationSettings: (enabled: boolean) => Promise<void>

  // Assistant actions
  loadAssistants: () => Promise<void>
  createAssistant: (data: Partial<Assistant>) => Promise<Assistant>
  updateAssistant: (id: string, data: Partial<Assistant>) => Promise<Assistant>
  deleteAssistant: (id: string) => Promise<void>

  // Proxy settings
  loadProxySettings: () => Promise<void>
  updateProxySettings: (settings: ProxySettings) => Promise<void>

  // Language settings
  updateDefaultLanguage: (language: string) => Promise<void>

  // Utility actions
  clearError: () => void
}

export const useAdminStore = create<AdminState>()(
  subscribeWithSelector((set, get) => ({
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

    // User actions
    loadUsers: async () => {
      try {
        set({ loadingUsers: true, error: null })

        const response = await ApiClient.Admin.listUsers({
          page: 1,
          per_page: 50,
        })

        set({
          users: response.users,
          loadingUsers: false,
        })
      } catch (error) {
        set({
          error:
            error instanceof Error ? error.message : 'Failed to load users',
          loadingUsers: false,
        })
        throw error
      }
    },

    updateUser: async (id: string, data: Partial<User>) => {
      try {
        set({ updating: true, error: null })

        const user = await ApiClient.Admin.updateUser({ user_id: id, ...data })

        set(state => ({
          users: state.users.map(u => (u.id === id ? user : u)),
          updating: false,
        }))

        return user
      } catch (error) {
        set({
          error:
            error instanceof Error ? error.message : 'Failed to update user',
          updating: false,
        })
        throw error
      }
    },

    resetUserPassword: async (id: string, newPassword: string) => {
      try {
        set({ updating: true, error: null })

        await ApiClient.Admin.resetPassword({
          user_id: id,
          new_password: newPassword,
        })

        set({ updating: false })
      } catch (error) {
        set({
          error:
            error instanceof Error ? error.message : 'Failed to reset password',
          updating: false,
        })
        throw error
      }
    },

    toggleUserActive: async (id: string) => {
      try {
        set({ updating: true, error: null })

        await ApiClient.Admin.toggleUserActive({ user_id: id })

        set(state => ({
          users: state.users.map(u =>
            u.id === id ? { ...u, is_active: !u.is_active } : u,
          ),
          updating: false,
        }))
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to toggle user status',
          updating: false,
        })
        throw error
      }
    },

    // User Group actions
    loadGroups: async () => {
      try {
        set({ loadingGroups: true, error: null })

        const response = await ApiClient.Admin.listGroups({
          page: 1,
          per_page: 50,
        })

        set({
          groups: response.groups,
          loadingGroups: false,
        })
      } catch (error) {
        set({
          error:
            error instanceof Error ? error.message : 'Failed to load groups',
          loadingGroups: false,
        })
        throw error
      }
    },

    createGroup: async (data: CreateUserGroupRequest) => {
      try {
        set({ creating: true, error: null })

        const group = await ApiClient.Admin.createGroup(data)

        set(state => ({
          groups: [...state.groups, group],
          creating: false,
        }))

        return group
      } catch (error) {
        set({
          error:
            error instanceof Error ? error.message : 'Failed to create group',
          creating: false,
        })
        throw error
      }
    },

    updateGroup: async (id: string, data: Partial<UserGroup>) => {
      try {
        set({ updating: true, error: null })

        const group = await ApiClient.Admin.updateGroup({
          group_id: id,
          ...data,
        })

        set(state => ({
          groups: state.groups.map(g => (g.id === id ? group : g)),
          updating: false,
        }))

        return group
      } catch (error) {
        set({
          error:
            error instanceof Error ? error.message : 'Failed to update group',
          updating: false,
        })
        throw error
      }
    },

    deleteGroup: async (id: string) => {
      try {
        set({ deleting: true, error: null })

        await ApiClient.Admin.deleteGroup({ group_id: id })

        set(state => ({
          groups: state.groups.filter(g => g.id !== id),
          deleting: false,
        }))
      } catch (error) {
        set({
          error:
            error instanceof Error ? error.message : 'Failed to delete group',
          deleting: false,
        })
        throw error
      }
    },

    loadGroupMembers: async (groupId: string) => {
      try {
        set({ loadingGroupMembers: true, error: null })

        const response = await ApiClient.Admin.getGroupMembers({
          group_id: groupId,
          page: 1,
          per_page: 50,
        })

        set({
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
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to load group members',
          loadingGroupMembers: false,
        })
        throw error
      }
    },

    assignUserToGroup: async (userId: string, groupId: string) => {
      try {
        set({ updating: true, error: null })

        await ApiClient.Admin.assignUserToGroup({
          user_id: userId,
          group_id: groupId,
        })

        // Reload group members if we're viewing this group
        const { currentGroupMembers } = get()
        if (currentGroupMembers.length > 0) {
          await get().loadGroupMembers(groupId)
        }

        set({ updating: false })
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to assign user to group',
          updating: false,
        })
        throw error
      }
    },

    removeUserFromGroup: async (userId: string, groupId: string) => {
      try {
        set({ updating: true, error: null })

        await ApiClient.Admin.removeUserFromGroup({
          user_id: userId,
          group_id: groupId,
        })

        // Remove from current group members
        set(state => ({
          currentGroupMembers: state.currentGroupMembers.filter(
            m => m.id !== userId,
          ),
          updating: false,
        }))
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to remove user from group',
          updating: false,
        })
        throw error
      }
    },

    // Registration settings
    loadUserRegistrationSettings: async () => {
      try {
        set({ loadingRegistrationSettings: true, error: null })

        const { enabled } = await ApiClient.Admin.getUserRegistrationStatus()

        set({
          userRegistrationEnabled: enabled,
          loadingRegistrationSettings: false,
        })
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to load registration settings',
          loadingRegistrationSettings: false,
        })
        throw error
      }
    },

    updateUserRegistrationSettings: async (enabled: boolean) => {
      try {
        set({ updating: true, error: null })

        await ApiClient.Admin.updateUserRegistrationStatus({ enabled })

        set({
          userRegistrationEnabled: enabled,
          updating: false,
        })
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to update registration settings',
          updating: false,
        })
        throw error
      }
    },

    // Assistant actions
    loadAssistants: async () => {
      try {
        set({ loading: true, error: null })
        const response = await ApiClient.Assistant.list({
          page: 1,
          per_page: 50,
        })
        set({
          assistants: response.assistants,
          loading: false,
        })
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to load assistants',
          loading: false,
        })
        throw error
      }
    },

    createAssistant: async (data: Partial<Assistant>) => {
      try {
        set({ creating: true, error: null })
        const assistant = await ApiClient.Assistant.create(data as any)
        set(state => ({
          assistants: [...state.assistants, assistant],
          creating: false,
        }))
        return assistant
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to create assistant',
          creating: false,
        })
        throw error
      }
    },

    updateAssistant: async (id: string, data: Partial<Assistant>) => {
      try {
        set({ updating: true, error: null })
        const assistant = await ApiClient.Assistant.update({
          assistant_id: id,
          ...data,
        })
        set(state => ({
          assistants: state.assistants.map(a => (a.id === id ? assistant : a)),
          updating: false,
        }))
        return assistant
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to update assistant',
          updating: false,
        })
        throw error
      }
    },

    deleteAssistant: async (id: string) => {
      try {
        set({ deleting: true, error: null })
        await ApiClient.Assistant.delete({ assistant_id: id })
        set(state => ({
          assistants: state.assistants.filter(a => a.id !== id),
          deleting: false,
        }))
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to delete assistant',
          deleting: false,
        })
        throw error
      }
    },

    // Proxy settings
    loadProxySettings: async () => {
      try {
        set({ loadingProxySettings: true, error: null })

        const settings = await ApiClient.Admin.getProxySettings()

        set({
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
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to load proxy settings',
          loadingProxySettings: false,
        })
        throw error
      }
    },

    updateProxySettings: async (settings: ProxySettings) => {
      try {
        set({ updating: true, error: null })

        await ApiClient.Admin.updateProxySettings(settings)

        set({
          proxySettings: settings,
          updating: false,
        })
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to update proxy settings',
          updating: false,
        })
        throw error
      }
    },

    updateDefaultLanguage: async (language: string) => {
      try {
        set({ updating: true, error: null })

        await ApiClient.Admin.updateDefaultLanguage({ language })

        set({ updating: false })
      } catch (error) {
        set({
          error:
            error instanceof Error
              ? error.message
              : 'Failed to update default language',
          updating: false,
        })
        throw error
      }
    },

    clearError: () => {
      set({ error: null })
    },
  })),
)
