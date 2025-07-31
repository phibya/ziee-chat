import { useEffect } from 'react'
import {
  BrowserRouter as Router,
  Navigate,
  Route,
  Routes,
} from 'react-router-dom'
import { isDesktopApp } from './api/core'
import { AuthGuard } from './components/Auth'
import { AppLayout } from './components/Layout/AppLayout'
import { AssistantsPage } from './components/Pages/Assistants'
import { ChatHistoryPage } from './components/Pages/ChatHistoryPage'
import { ChatPage } from './components/Pages/ChatPage'
import { HubPage } from './components/Pages/Hub'
import { ProjectDetailsPage } from './components/Pages/Projects/ProjectDetailsPage.tsx'
import { ProjectsPage } from './components/Pages/Projects/ProjectsPage.tsx'
import {
  AdminAppearanceSettings,
  AdminAssistantsSettings,
  AdminGeneralSettings,
  AppearanceSettings,
  DocumentExtractionSettings,
  ExtensionsSettings,
  GeneralSettings,
  HardwareSettings,
  HttpsProxySettings,
  ModelRepositorySettings,
  PrivacySettings,
  ProvidersSettings,
  RAGProvidersSettings,
  RAGRepositoriesSettings,
  ShortcutsSettings,
  UserGroupsSettings,
  UsersSettings,
} from './components/Pages/Settings'
import { SettingsPage } from './components/Pages/SettingsPage'
import { Permission, usePermissions } from './permissions'
import { ThemeProvider } from './providers/ThemeProvider'
import './i18n'
import '@ant-design/v5-patch-for-react-19'
import { useTranslation } from 'react-i18next'
import './store/startup'
import { useUserAppearanceLanguage } from './store'
import { App as AntdApp } from 'antd'

function App() {
  const { i18n } = useTranslation()
  const language = useUserAppearanceLanguage()
  const { hasPermission } = usePermissions()

  // // Update language when settings change
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
                <Route path="/" element={<ChatPage />} />
                <Route
                  path="/conversation/:conversationId"
                  element={<ChatPage />}
                />
                <Route path="/assistants" element={<AssistantsPage />} />
                <Route path="/chat-history" element={<ChatHistoryPage />} />
                <Route path="/projects" element={<ProjectsPage />} />
                <Route
                  path="/projects/:projectId"
                  element={<ProjectDetailsPage />}
                />
                <Route path="/hub" element={<HubPage />} />
                <Route path="/hub/:activeTab" element={<HubPage />} />
                <Route path="/settings" element={<SettingsPage />}>
                  <Route
                    index
                    element={<Navigate to="/settings/general" replace />}
                  />
                  <Route path="general" element={<GeneralSettings />} />
                  <Route path="appearance" element={<AppearanceSettings />} />
                  <Route path="privacy" element={<PrivacySettings />} />
                  {/* Providers: Main settings for desktop, Admin section for web */}
                  {(isDesktopApp ||
                    (!isDesktopApp &&
                      hasPermission(Permission.config.providers.read))) && (
                    <>
                      <Route path="providers" element={<ProvidersSettings />} />
                      <Route
                        path="providers/:provider_id"
                        element={<ProvidersSettings />}
                      />
                    </>
                  )}
                  {(isDesktopApp ||
                    hasPermission(Permission.config.repositories.read)) && (
                    <Route
                      path="repositories"
                      element={<ModelRepositorySettings />}
                    />
                  )}
                  {/* RAG Providers: Similar permissions to providers */}
                  {(isDesktopApp ||
                    (!isDesktopApp &&
                      hasPermission(Permission.config.providers.read))) && (
                    <>
                      <Route path="rag-providers" element={<RAGProvidersSettings />} />
                      <Route
                        path="rag-providers/:provider_id"
                        element={<RAGProvidersSettings />}
                      />
                    </>
                  )}
                  {/* RAG Repositories: Similar permissions to repositories */}
                  {(isDesktopApp ||
                    hasPermission(Permission.config.repositories.read)) && (
                    <Route
                      path="rag-repositories"
                      element={<RAGRepositoriesSettings />}
                    />
                  )}
                  <Route path="shortcuts" element={<ShortcutsSettings />} />
                  <Route path="hardware" element={<HardwareSettings />} />
                  <Route path="https-proxy" element={<HttpsProxySettings />} />
                  <Route path="extensions" element={<ExtensionsSettings />} />
                  {!isDesktopApp && hasPermission(Permission.users.read) && (
                    <Route path="users" element={<UsersSettings />} />
                  )}
                  {!isDesktopApp && hasPermission(Permission.groups.read) && (
                    <Route
                      path="user-groups"
                      element={<UserGroupsSettings />}
                    />
                  )}
                  {!isDesktopApp &&
                    hasPermission(Permission.config.experimental.edit) && (
                      <Route
                        path="admin-general"
                        element={<AdminGeneralSettings />}
                      />
                    )}
                  {!isDesktopApp &&
                    hasPermission(Permission.config.experimental.edit) && (
                      <Route
                        path="admin-appearance"
                        element={<AdminAppearanceSettings />}
                      />
                    )}
                  {!isDesktopApp &&
                    hasPermission(
                      Permission.config.documentExtraction.read,
                    ) && (
                      <Route
                        path="admin-document-extraction"
                        element={<DocumentExtractionSettings />}
                      />
                    )}
                  {!isDesktopApp &&
                    hasPermission(Permission.config.assistants.read) && (
                      <Route
                        path="admin-assistants"
                        element={<AdminAssistantsSettings />}
                      />
                    )}
                </Route>
                <Route path="*" element={<Navigate to="/" replace />} />
              </Routes>
            </AppLayout>
          </AntdApp>
        </Router>
      </AuthGuard>
    </ThemeProvider>
  )
}

export default App
