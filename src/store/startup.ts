// operations to run after user authentication

import { hasPermission, Permission } from '../permissions'
import { useAuthStore } from './auth.ts'
import { initializeDownloadTracking } from './admin/modelDownload.ts'
import { initializeUserSettings } from './settings.ts'

useAuthStore.subscribe(
  state => state.user,
  user => {
    if (!user) return
    initializeUserSettings()
    if (hasPermission(user, Permission.config.providers.read)) {
      initializeDownloadTracking()
    }
  },
)
