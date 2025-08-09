import { useEffect } from 'react'
import { BrowserRouter as Router, Route, Routes } from 'react-router-dom'
import { AuthGuard } from './components2/Auth'
import { ThemeProvider } from './components2/Providers/ThemeProvider'
import { AppLayout } from './components2/Layout/AppLayout'
import { App as AntdApp } from 'antd'
import { useTranslation } from 'react-i18next'
import { useUserAppearanceLanguage } from './store'
import { ProjectsPage } from './components2/Pages/Projects/ProjectsPage'
import { ProjectDetailsPage } from './components2/Pages/Projects/ProjectDetailsPage'
import {
  NewChatInterface,
  ExistingChatInterface,
} from './components2/Pages/Chat'
import { ChatHistoryPage } from './components2/Pages/ChatHistoryPage'
import { HubPage } from './components2/Pages/Hub/HubPage'
import { AssistantsPage } from './components2/Pages/Assistants/AssistantsPage'
import { SettingsPage } from './components2/Pages/SettingsPage'
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
  AdminGeneralSettings,
} from './components2/Pages/Settings'
import './i18n'
import '@ant-design/v5-patch-for-react-19'
import './store/startup'

function App2() {
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
            <AppLayout>
              <Routes>
                <Route path="/" element={<NewChatInterface />} />
                <Route
                  path="/conversation/:conversationId"
                  element={<ExistingChatInterface />}
                />
                <Route path="/conversations" element={<ChatHistoryPage />} />
                <Route path="/projects" element={<ProjectsPage />} />
                <Route
                  path="/projects/:projectId"
                  element={<ProjectDetailsPage />}
                />
                <Route path="/hub/:activeTab?" element={<HubPage />} />
                <Route path="/assistants" element={<AssistantsPage />} />
                <Route path="/settings" element={<SettingsPage />}>
                  <Route path="general" element={<GeneralSettings />} />
                  <Route path="appearance" element={<AppearanceSettings />} />
                  <Route path="privacy" element={<PrivacySettings />} />
                  <Route path="providers" element={<ProvidersSettings />} />
                  <Route
                    path="repositories"
                    element={<ModelRepositorySettings />}
                  />
                  <Route
                    path="rag-providers"
                    element={<RAGProvidersSettings />}
                  />
                  <Route
                    path="rag-repositories"
                    element={<RAGRepositoriesSettings />}
                  />
                  <Route path="shortcuts" element={<ShortcutsSettings />} />
                  <Route path="hardware" element={<HardwareSettings />} />
                  <Route path="https-proxy" element={<HttpsProxySettings />} />
                  <Route path="extensions" element={<ExtensionsSettings />} />
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
                  <Route path="users" element={<UsersSettings />} />
                  <Route path="user-groups" element={<UserGroupsSettings />} />
                </Route>
              </Routes>
            </AppLayout>
          </AntdApp>
        </Router>
      </AuthGuard>
    </ThemeProvider>
  )
}

export default App2
