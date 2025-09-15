// operations to run after user authentication

import { useAuthStore } from './auth.ts'
import { initializeUserSettings } from './settings.ts'

useAuthStore.subscribe(
  state => state.user,
  user => {
    if (!user) return
    initializeUserSettings()
  },
)
