import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import type { CreateUserRequest, LoginRequest, User } from '../api/enpoints'
import { ApiClient } from '../api/client.ts'

interface AuthState {
  user: User | null
  token: string | null
  isAuthenticated: boolean
  isLoading: boolean
  needsSetup: boolean
  isDesktop: boolean
  error: string | null

  // Actions
  login: (credentials: LoginRequest) => Promise<void>
  logout: () => Promise<void>
  register: (userData: CreateUserRequest) => Promise<void>
  setupApp: (userData: CreateUserRequest) => Promise<void>
  checkInitStatus: () => Promise<void>
  getCurrentUser: () => Promise<void>
  clearError: () => void
}

const defaultState = {
  user: null,
  token: null,
  isAuthenticated: false,
  isLoading: false,
  needsSetup: false,
  isDesktop: false,
  error: null,
} as const as AuthState

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      ...defaultState,

      login: async (credentials: LoginRequest) => {
        set({ isLoading: true, error: null })
        try {
          const { token, user } = await ApiClient.Auth.login(credentials)

          set({
            user,
            token,
            isAuthenticated: true,
            isLoading: false,
            error: null,
          })
        } catch (error) {
          set({
            error: error instanceof Error ? error.message : 'Login failed',
            isLoading: false,
            isAuthenticated: false,
            token: null,
            user: null,
          })
          throw error
        }
      },

      logout: async () => {
        set({ isLoading: true, error: null })
        try {
          const { token } = get()
          if (token) {
            // Call logout API to invalidate token on server
            await ApiClient.Auth.logout()
          }

          set({
            user: null,
            token: null,
            isAuthenticated: false,
            isLoading: false,
            error: null,
          })
        } catch (error) {
          // Even if logout fails on server, clear local state
          set({
            user: null,
            token: null,
            isAuthenticated: false,
            isLoading: false,
            error: null,
          })
        }
      },

      register: async (userData: CreateUserRequest) => {
        set({ isLoading: true, error: null })
        try {
          const { token, user } = await ApiClient.Auth.register(userData)

          set({
            user,
            token,
            isAuthenticated: true,
            isLoading: false,
            error: null,
          })
        } catch (error) {
          set({
            error:
              error instanceof Error ? error.message : 'Registration failed',
            isLoading: false,
          })
          throw error
        }
      },

      setupApp: async (userData: CreateUserRequest) => {
        set({ isLoading: true, error: null })
        try {
          const { token, user } = await ApiClient.Auth.setup(userData)

          set({
            user,
            token,
            isAuthenticated: true,
            isLoading: false,
            needsSetup: false,
            error: null,
          })
        } catch (error) {
          set({
            error: error instanceof Error ? error.message : 'Setup failed',
            isLoading: false,
          })
          throw error
        }
      },

      checkInitStatus: async () => {
        set({ isLoading: true, error: null })

        try {
          const { needs_setup, is_desktop } = await ApiClient.Auth.init()

          set({
            needsSetup: needs_setup,
            isDesktop: is_desktop,
            isLoading: false,
          })

          // For desktop app, automatically attempt login
          if (is_desktop) {
            try {
              await get().login({
                username_or_email: 'admin',
                password: 'admin',
              })
            } catch (error) {
              // Desktop auto-login failed, but that's okay
              console.warn('Desktop auto-login failed:', error)
            }
          }
        } catch (error) {
          set({
            error:
              error instanceof Error
                ? error.message
                : 'Failed to check init status',
            isLoading: false,
          })
        }
      },

      getCurrentUser: async () => {
        const { token } = get()
        if (!token) return

        set({ isLoading: true, error: null })
        try {
          const user = await ApiClient.Auth.me()

          set({
            user,
            isAuthenticated: true,
            isLoading: false,
            error: null,
          })
        } catch (error) {
          set({
            error:
              error instanceof Error
                ? error.message
                : 'Failed to get current user',
            isLoading: false,
            isAuthenticated: false,
            token: null,
            user: null,
          })
        }
      },

      clearError: () => set({ error: null }),
    }),
    {
      name: 'auth-storage',
      version: 1,
      partialize: state => ({
        user: state.user,
        token: state.token,
        isAuthenticated: state.isAuthenticated,
        isDesktop: state.isDesktop,
      }),
    },
  ),
)
