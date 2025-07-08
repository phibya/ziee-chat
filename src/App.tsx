import { useEffect } from 'react'
import {
  BrowserRouter as Router,
  Navigate,
  Route,
  Routes,
} from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { ThemeProvider } from './providers/ThemeProvider'
import { AuthGuard } from './components/Auth'
import { AppLayout } from './components/Layout/AppLayout'
import { ChatPage } from './components/Pages/ChatPage'
import { AssistantsPage } from './components/Pages/AssistantsPage'
import { HubPage } from './components/Pages/HubPage'
import { SettingsPage } from './components/Pages/SettingsPage'
import { ModelsPage } from './components/Pages/ModelsPage'
import {
  AppearanceSettings,
  ExtensionsSettings,
  GeneralSettings,
  HardwareSettings,
  HttpsProxySettings,
  LocalApiServerSettings,
  ModelProvidersSettings,
  PrivacySettings,
  ShortcutsSettings,
  UserGroupsSettings,
  UsersSettings,
} from './components/Pages/Settings'
import { useSettingsStore } from './store/settings'
import { usePermissions } from './hooks/usePermissions'
import { PermissionKeys } from './api/enpoints'
import { isDesktopApp } from './api/core'
import './i18n'
import '@ant-design/v5-patch-for-react-19'

function App() {
  const { i18n } = useTranslation()
  const { language } = useSettingsStore()
  const { hasPermission } = usePermissions()

  // Update language when settings change
  useEffect(() => {
    if (i18n.language !== language) {
      i18n.changeLanguage(language)
    }
  }, [language, i18n])

  return (
    <ThemeProvider>
      <AuthGuard>
        <Router>
          <AppLayout>
            <Routes>
              <Route path="/" element={<ChatPage />} />
              <Route path="/assistants" element={<AssistantsPage />} />
              <Route path="/hub" element={<HubPage />} />
              <Route path="/models" element={<ModelsPage />} />
              <Route path="/settings" element={<SettingsPage />}>
                <Route
                  index
                  element={<Navigate to="/settings/general" replace />}
                />
                <Route path="general" element={<GeneralSettings />} />
                <Route path="appearance" element={<AppearanceSettings />} />
                <Route path="privacy" element={<PrivacySettings />} />
                <Route
                  path="model-providers"
                  element={<ModelProvidersSettings />}
                />
                <Route path="shortcuts" element={<ShortcutsSettings />} />
                <Route path="hardware" element={<HardwareSettings />} />
                <Route
                  path="local-api-server"
                  element={<LocalApiServerSettings />}
                />
                <Route path="https-proxy" element={<HttpsProxySettings />} />
                <Route path="extensions" element={<ExtensionsSettings />} />
                {!isDesktopApp &&
                  hasPermission(PermissionKeys.USERS_READ) && (
                    <Route path="users" element={<UsersSettings />} />
                  )}
                {!isDesktopApp &&
                  hasPermission(PermissionKeys.GROUPS_READ) && (
                    <Route
                      path="user-groups"
                      element={<UserGroupsSettings />}
                    />
                  )}
              </Route>
              <Route path="*" element={<Navigate to="/" replace />} />
            </Routes>
          </AppLayout>
        </Router>
      </AuthGuard>
    </ThemeProvider>
  )
}

export default App
