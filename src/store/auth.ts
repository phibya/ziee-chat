import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import type { CreateUserRequest, LoginRequest, User } from '../types'
import { ApiClient } from '../api/client.ts'

interface AuthState {
  user: User | null
  token: string | null
  isAuthenticated: boolean
  isLoading: boolean
  needsSetup: boolean
  isDesktop: boolean
  error: string | null
}

const defaultState: AuthState = {
  user: null,
  token: null,
  isAuthenticated: false,
  isLoading: false,
  needsSetup: false,
  isDesktop: false,
  error: null,
}

export const useAuthStore = create<AuthState>()(
  persist((): AuthState => defaultState, {
    name: 'auth-storage',
  }),
)

// Auth actions
export const authenticateUser = async (credentials: LoginRequest): Promise<void> => {
  useAuthStore.setState({ isLoading: true, error: null })
  try {
    const { token, user } = await ApiClient.Auth.login(credentials)

    useAuthStore.setState({
      user,
      token,
      isAuthenticated: true,
      isLoading: false,
      error: null,
    })
  } catch (error) {
    useAuthStore.setState({
      error: error instanceof Error ? error.message : 'Login failed',
      isLoading: false,
      isAuthenticated: false,
      token: null,
      user: null,
    })
    throw error
  }
}

export const logoutUser = async (): Promise<void> => {
  useAuthStore.setState({ isLoading: true, error: null })
  try {
    const { token } = useAuthStore.getState()
    if (token) {
      // Call logout API to invalidate token on server
      await ApiClient.Auth.logout()
    }

    useAuthStore.setState({
      user: null,
      token: null,
      isAuthenticated: false,
      isLoading: false,
      error: null,
    })
  } catch {
    // Even if logout fails on server, clear local state
    useAuthStore.setState({
      user: null,
      token: null,
      isAuthenticated: false,
      isLoading: false,
      error: null,
    })
  }
}

export const registerNewUser = async (userData: CreateUserRequest): Promise<void> => {
  useAuthStore.setState({ isLoading: true, error: null })
  try {
    const { token, user } = await ApiClient.Auth.register(userData)

    useAuthStore.setState({
      user,
      token,
      isAuthenticated: true,
      isLoading: false,
      error: null,
    })
  } catch (error) {
    useAuthStore.setState({
      error: error instanceof Error ? error.message : 'Registration failed',
      isLoading: false,
    })
    throw error
  }
}

export const setupInitialAdminUser = async (userData: CreateUserRequest): Promise<void> => {
  useAuthStore.setState({ isLoading: true, error: null })
  try {
    const { token, user } = await ApiClient.Auth.setup(userData)

    useAuthStore.setState({
      user,
      token,
      isAuthenticated: true,
      isLoading: false,
      needsSetup: false,
      error: null,
    })
  } catch (error) {
    useAuthStore.setState({
      error: error instanceof Error ? error.message : 'Setup failed',
      isLoading: false,
    })
    throw error
  }
}

export const checkApplicationInitializationStatus = async (): Promise<void> => {
  useAuthStore.setState({ isLoading: true, error: null })
  try {
    const response = await ApiClient.Auth.init()

    useAuthStore.setState({
      needsSetup: response.needs_setup,
      isDesktop: response.is_desktop,
      isLoading: false,
    })
  } catch (error) {
    useAuthStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to check initialization status',
      isLoading: false,
    })
    throw error
  }
}

export const fetchCurrentUserProfile = async (): Promise<void> => {
  useAuthStore.setState({ isLoading: true, error: null })
  try {
    const user = await ApiClient.Auth.me()

    useAuthStore.setState({
      user,
      isAuthenticated: true,
      isLoading: false,
    })
  } catch (error) {
    useAuthStore.setState({
      error:
        error instanceof Error ? error.message : 'Failed to get current user',
      isLoading: false,
      isAuthenticated: false,
      user: null,
      token: null,
    })
    throw error
  }
}

export const clearAuthenticationError = (): void => {
  useAuthStore.setState({ error: null })
}
