import { useEffect } from 'react'
import {
  BrowserRouter as Router,
  Routes,
  Route,
  Navigate,
} from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { ThemeProvider } from './providers/ThemeProvider'
import { AppLayout } from './components/Layout/AppLayout'
import { ChatPage } from './components/Pages/ChatPage'
import { AssistantsPage } from './components/Pages/AssistantsPage'
import { HubPage } from './components/Pages/HubPage'
import { SettingsPage } from './components/Pages/SettingsPage'
import { ModelsPage } from './components/Pages/ModelsPage'
import {
  GeneralSettings,
  AppearanceSettings,
  PrivacySettings,
  ModelProvidersSettings,
  ShortcutsSettings,
  HardwareSettings,
  LocalApiServerSettings,
  HttpsProxySettings,
  ExtensionsSettings,
} from './components/Pages/Settings'
import { useSettingsStore } from './store/settings'
import './i18n'
import '@ant-design/v5-patch-for-react-19'

function App() {
  const { i18n } = useTranslation()
  const { language } = useSettingsStore()

  // Update language when settings change
  useEffect(() => {
    if (i18n.language !== language) {
      i18n.changeLanguage(language)
    }
  }, [language, i18n])

  return (
    <ThemeProvider>
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
            </Route>
            <Route path="*" element={<Navigate to="/" replace />} />
          </Routes>
        </AppLayout>
      </Router>
    </ThemeProvider>
  )
}

export default App
