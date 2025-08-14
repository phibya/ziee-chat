import { useEffect } from 'react'
import { BrowserRouter as Router, Route, Routes } from 'react-router-dom'
import { AuthGuard } from './components/Auth'
import { ThemeProvider } from './components/Providers/ThemeProvider'
import { AppLayout } from './components/Layout/AppLayout'
import { App as AntdApp } from 'antd'
import { useTranslation } from 'react-i18next'
import { useUserAppearanceLanguage } from './store'
import { ProjectsPage } from './components/Pages/Projects/ProjectsPage'
import { ProjectDetailsPage } from './components/Pages/Projects/ProjectDetailsPage'
import {
  NewChatInterface,
  ExistingChatInterface,
} from './components/Pages/Chat'
import { ChatHistoryPage } from './components/Pages/ChatHistoryPage'
import { HubPage } from './components/Pages/Hub/HubPage'
import { AssistantsPage } from './components/Pages/Assistants'
import { SettingsPage } from './components/Pages/SettingsPage'
import { HardwareMonitor } from './components/Pages/HardwareMonitor'
import {
  GeneralSettings,
  AppearanceSettings,
  PrivacySettings,
  ProvidersSettings,
  ModelRepositorySettings,
  RAGProvidersSettings,
  RAGRepositoriesSettings,
  ShortcutsSettings,
  HardwareSettings,
  HttpsProxySettings,
  ExtensionsSettings,
  UserGroupsSettings,
  UsersSettings,
  AdminAppearanceSettings,
  AdminAssistantsSettings,
  EnginesSettings,
  AdminGeneralSettings,
} from './components/Pages/Settings'
import './i18n'
import '@ant-design/v5-patch-for-react-19'
import './store/startup'

function App() {
  const { i18n } = useTranslation()
  const language = useUserAppearanceLanguage()

  // Update language when settings change
  useEffect(() => {
    if (i18n.language !== language) {
      i18n.changeLanguage(language)
    }
  }, [language, i18n.language])

  return (
    <ThemeProvider>
      <AuthGuard>
        <Router>
          <AntdApp>
            <Routes>
              {/* Hardware Monitor Route - Outside AppLayout for popup usage */}
              <Route path="/hardware-monitor" element={<HardwareMonitor />} />

              {/* Main App Routes - Inside AppLayout */}
              <Route
                path="/*"
                element={
                  <AppLayout>
                    <Routes>
                      <Route path="/" element={<NewChatInterface />} />
                      <Route
                        path="/conversation/:conversationId"
                        element={<ExistingChatInterface />}
                      />
                      <Route
                        path="/conversations"
                        element={<ChatHistoryPage />}
                      />
                      <Route path="/projects" element={<ProjectsPage />} />
                      <Route
                        path="/projects/:projectId"
                        element={<ProjectDetailsPage />}
                      />
                      <Route path="/hub/:activeTab?" element={<HubPage />} />
                      <Route path="/assistants" element={<AssistantsPage />} />
                      <Route path="/settings" element={<SettingsPage />}>
                        <Route path="" element={<GeneralSettings />} />
                        <Route path="general" element={<GeneralSettings />} />
                        <Route
                          path="appearance"
                          element={<AppearanceSettings />}
                        />
                        <Route path="privacy" element={<PrivacySettings />} />
                        <Route
                          path="providers"
                          element={<ProvidersSettings />}
                        />
                        <Route
                          path="providers/:providerId"
                          element={<ProvidersSettings />}
                        />
                        <Route
                          path="repositories"
                          element={<ModelRepositorySettings />}
                        />
                        <Route
                          path="rag-providers"
                          element={<RAGProvidersSettings />}
                        />
                        <Route
                          path="rag-providers/:providerId"
                          element={<RAGProvidersSettings />}
                        />
                        <Route
                          path="rag-repositories"
                          element={<RAGRepositoriesSettings />}
                        />
                        <Route
                          path="shortcuts"
                          element={<ShortcutsSettings />}
                        />
                        <Route path="hardware" element={<HardwareSettings />} />
                        <Route
                          path="https-proxy"
                          element={<HttpsProxySettings />}
                        />
                        <Route
                          path="extensions"
                          element={<ExtensionsSettings />}
                        />
                        <Route
                          path="admin-general"
                          element={<AdminGeneralSettings />}
                        />
                        <Route
                          path="admin-appearance"
                          element={<AdminAppearanceSettings />}
                        />
                        <Route
                          path="admin-assistants"
                          element={<AdminAssistantsSettings />}
                        />
                        <Route
                          path="engines"
                          element={<EnginesSettings />}
                        />
                        <Route path="users" element={<UsersSettings />} />
                        <Route
                          path="user-groups"
                          element={<UserGroupsSettings />}
                        />
                      </Route>
                    </Routes>
                  </AppLayout>
                }
              />
            </Routes>
          </AntdApp>
        </Router>
      </AuthGuard>
    </ThemeProvider>
  )
}

export default App
