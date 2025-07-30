import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../../api/client.ts'
import { SupportedLanguage } from '../../types'

// Using API types now - User and UserGroup imported above


interface AdminState {


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
      creating: false,
      updating: false,
      deleting: false,
      error: null,
    }),
  ),
)



export const updateSystemDefaultLanguage = async (
  language: SupportedLanguage,
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
