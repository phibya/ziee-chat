import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../../api/client.ts'
import type { AuthProvider, TestConnectionResult } from '../../types'

interface AdminAuthProvidersState {
  // Data
  providers: AuthProvider[]
  isInitialized: boolean

  // Loading states
  loading: boolean
  creating: boolean
  updating: boolean
  deleting: boolean
  testing: Record<string, boolean> // Track testing state per provider

  // Error states
  error: string | null
  testErrors: Record<string, string> // Track test errors per provider

  __init__: {
    providers: () => Promise<void>
  }
}

export const useAdminAuthProvidersStore = create<AdminAuthProvidersState>()(
  subscribeWithSelector(
    (): AdminAuthProvidersState => ({
      // Initial state
      providers: [],
      isInitialized: false,
      loading: false,
      creating: false,
      updating: false,
      deleting: false,
      testing: {},
      error: null,
      testErrors: {},
      __init__: {
        providers: async () => loadAuthProviders(),
      },
    }),
  ),
)

// Provider actions
export const loadAuthProviders = async (): Promise<void> => {
  const state = useAdminAuthProvidersStore.getState()
  if (state.isInitialized || state.loading) {
    return
  }

  try {
    useAdminAuthProvidersStore.setState({ loading: true, error: null })
    const providers = await ApiClient.Admin.listAuthProviders()

    useAdminAuthProvidersStore.setState({
      providers,
      isInitialized: true,
      loading: false,
    })
  } catch (error) {
    useAdminAuthProvidersStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to load auth providers',
      loading: false,
    })
    throw error
  }
}

export const getAuthProvider = async (
  providerId: string,
): Promise<AuthProvider | undefined> => {
  try {
    const provider = await ApiClient.Admin.getAuthProvider({
      provider_id: providerId,
    })
    return provider
  } catch (error) {
    console.error('Failed to get auth provider:', error)
    throw error
  }
}

export const createAuthProvider = async (data: {
  name: string
  provider_type: string
  enabled?: boolean
  priority?: number
  config: Record<string, unknown>
  mapping_rules?: Record<string, unknown>
}): Promise<AuthProvider | undefined> => {
  const state = useAdminAuthProvidersStore.getState()
  if (state.creating) {
    return
  }

  try {
    useAdminAuthProvidersStore.setState({ creating: true, error: null })
    const newProvider = await ApiClient.Admin.createAuthProvider({
      ...data,
      enabled: data.enabled ?? true,
      priority: data.priority ?? 0,
      mapping_rules: data.mapping_rules || {},
    })

    useAdminAuthProvidersStore.setState(state => ({
      providers: [...state.providers, newProvider],
      creating: false,
    }))

    return newProvider
  } catch (error) {
    useAdminAuthProvidersStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to create auth provider',
      creating: false,
    })
    throw error
  }
}

export const updateAuthProvider = async (
  providerId: string,
  data: {
    name?: string
    enabled?: boolean
    priority?: number
    config?: Record<string, unknown>
    mapping_rules?: Record<string, unknown>
  },
): Promise<void> => {
  const state = useAdminAuthProvidersStore.getState()
  if (state.updating) {
    return
  }

  try {
    useAdminAuthProvidersStore.setState({ updating: true, error: null })
    const updatedProvider = await ApiClient.Admin.updateAuthProvider({
      provider_id: providerId,
      ...data,
    })

    useAdminAuthProvidersStore.setState(state => ({
      providers: state.providers.map(p =>
        p.id === providerId ? updatedProvider : p,
      ),
      updating: false,
    }))
  } catch (error) {
    useAdminAuthProvidersStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to update auth provider',
      updating: false,
    })
    throw error
  }
}

export const deleteAuthProvider = async (providerId: string): Promise<void> => {
  const state = useAdminAuthProvidersStore.getState()
  if (state.deleting) {
    return
  }

  try {
    useAdminAuthProvidersStore.setState({ deleting: true, error: null })
    await ApiClient.Admin.deleteAuthProvider({ provider_id: providerId })

    useAdminAuthProvidersStore.setState(state => {
      // Clean up testing state and errors for this provider
      const { [providerId]: _removedTesting, ...restTesting } = state.testing
      const { [providerId]: _removedError, ...restTestErrors } =
        state.testErrors

      return {
        providers: state.providers.filter(p => p.id !== providerId),
        testing: restTesting,
        testErrors: restTestErrors,
        deleting: false,
      }
    })
  } catch (error) {
    useAdminAuthProvidersStore.setState({
      error:
        error instanceof Error
          ? error.message
          : 'Failed to delete auth provider',
      deleting: false,
    })
    throw error
  }
}

export const testAuthProviderConnection = async (
  providerId: string,
): Promise<TestConnectionResult> => {
  const state = useAdminAuthProvidersStore.getState()
  if (state.testing[providerId]) {
    throw new Error('Test already in progress')
  }

  try {
    useAdminAuthProvidersStore.setState(state => ({
      testing: { ...state.testing, [providerId]: true },
      testErrors: { ...state.testErrors, [providerId]: '' },
    }))

    const result = await ApiClient.Admin.testAuthProviderConnection({
      provider_id: providerId,
    })

    useAdminAuthProvidersStore.setState(state => ({
      testing: { ...state.testing, [providerId]: false },
    }))

    return result
  } catch (error) {
    useAdminAuthProvidersStore.setState(state => ({
      testErrors: {
        ...state.testErrors,
        [providerId]:
          error instanceof Error ? error.message : 'Test connection failed',
      },
      testing: { ...state.testing, [providerId]: false },
    }))
    throw error
  }
}

export const toggleProviderEnabled = async (
  providerId: string,
  enabled: boolean,
): Promise<void> => {
  await updateAuthProvider(providerId, { enabled })
}

// Utility actions
export const clearProvidersError = (): void => {
  useAdminAuthProvidersStore.setState({ error: null })
}

export const clearTestError = (providerId: string): void => {
  useAdminAuthProvidersStore.setState(state => ({
    testErrors: { ...state.testErrors, [providerId]: '' },
  }))
}

export const getProviderById = (id: string): AuthProvider | undefined => {
  return useAdminAuthProvidersStore.getState().providers.find(p => p.id === id)
}

export const getEnabledProviders = (): AuthProvider[] => {
  return useAdminAuthProvidersStore.getState().providers.filter(p => p.enabled)
}
