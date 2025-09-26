import { useEffect } from 'react'
import { BrowserRouter as Router, Route, Routes } from 'react-router-dom'
import { AuthGuard } from './components/Auth'
import { ThemeProvider } from './components/providers/ThemeProvider'
import { AppLayout } from './components/Layout/AppLayout'
import { useTranslation } from 'react-i18next'
import { useUserAppearanceLanguage } from './store'
import { ProjectsPage } from './components/Pages/Projects/ProjectsPage'
import { ProjectDetailsPage } from './components/Pages/Projects/ProjectDetailsPage'
import { RagsPage } from './components/Pages/Rags/RagsPage'
import { RagDetailsPage } from './components/Pages/Rags/RagDetailsPage'
import {
  ExistingChatInterface,
  NewChatInterface,
} from './components/Pages/Chat'
import { ChatHistoryPage } from './components/Pages/Chat/ChatHistoryPage.tsx'
import { HubPage } from './components/Pages/Hub/HubPage'
import { AssistantsPage } from './components/Pages/Assistants'
import { SettingsPage } from './components/Pages/SettingsPage'
import { HardwareMonitor } from './components/Pages/HardwareMonitor'
import { ApiProxyLogMonitor } from './components/Pages/Settings/ApiProxyServer/ApiProxyLogMonitor.tsx'
import {
  AdminAppearanceSettings,
  AdminAssistantsSettings,
  AdminGeneralSettings,
  ApiProxyServerSettings,
  AppearanceSettings,
  EnginesSettings,
  GeneralSettings,
  HardwareSettings,
  HttpsProxySettings,
  MCPServersSettings,
  ModelRepositorySettings,
  NgrokSettings,
  PrivacySettings,
  ProvidersSettings,
  RAGProvidersSettings,
  UserGroupsSettings,
  UsersSettings,
} from './components/Pages/Settings'
import { MCPAdminPage } from './components/Pages/Settings/MCPServers/Admin/MCPAdminPage'
import './i18n'
import '@ant-design/v5-patch-for-react-19'
import './store/startup'
import { PagePermissionGuard403 } from './components/Auth/PermissionGuard.tsx'
import { Permission } from './types'

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
          <Routes>
            {/* Hardware Monitor Route - Outside AppLayout for popup usage */}
            <Route path="/hardware-monitor" element={<HardwareMonitor />} />

            {/* API Proxy Log Monitor Route - Outside AppLayout for popup usage */}
            <Route
              path="/api-proxy-log-monitor"
              element={<ApiProxyLogMonitor />}
            />

            {/* Main App Routes - Inside AppLayout */}
            <Route
              path="/*"
              element={
                <AppLayout>
                  <Routes>
                    <Route
                      path="/"
                      element={
                        <PagePermissionGuard403
                          permissions={[Permission.ChatCreate]}
                        >
                          <NewChatInterface />
                        </PagePermissionGuard403>
                      }
                    />
                    <Route
                      path="/conversation/:conversationId"
                      element={
                        <PagePermissionGuard403
                          permissions={[Permission.ChatRead]}
                        >
                          <ExistingChatInterface />
                        </PagePermissionGuard403>
                      }
                    />
                    <Route
                      path="/conversations"
                      element={
                        <PagePermissionGuard403
                          permissions={[Permission.ChatRead]}
                        >
                          <ChatHistoryPage />
                        </PagePermissionGuard403>
                      }
                    />
                    <Route
                      path="/projects"
                      element={
                        <PagePermissionGuard403
                          permissions={[Permission.ProjectsRead]}
                        >
                          <ProjectsPage />
                        </PagePermissionGuard403>
                      }
                    />
                    <Route
                      path="/projects/:projectId"
                      element={
                        <PagePermissionGuard403
                          permissions={[Permission.ProjectsRead]}
                        >
                          <ProjectDetailsPage />
                        </PagePermissionGuard403>
                      }
                    />
                    <Route
                      path="/rags"
                      element={
                        <PagePermissionGuard403
                          permissions={[Permission.RagInstancesRead]}
                        >
                          <RagsPage />
                        </PagePermissionGuard403>
                      }
                    />
                    <Route
                      path="/rags/:ragInstanceId"
                      element={
                        <PagePermissionGuard403
                          permissions={[Permission.RagInstancesRead]}
                        >
                          <RagDetailsPage />
                        </PagePermissionGuard403>
                      }
                    />
                    <Route
                      path="/rags/:ragInstanceId/:tab"
                      element={
                        <PagePermissionGuard403
                          permissions={[Permission.RagInstancesRead]}
                        >
                          <RagDetailsPage />
                        </PagePermissionGuard403>
                      }
                    />
                    <Route
                      path="/hub/:activeTab?"
                      element={
                        <PagePermissionGuard403
                          permissions={[
                            Permission.HubModelsRead,
                            Permission.HubAssistantsRead,
                          ]}
                          match={'any'}
                        >
                          <HubPage />
                        </PagePermissionGuard403>
                      }
                    />
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
                        element={
                          <PagePermissionGuard403
                            permissions={[Permission.ProvidersRead]}
                          >
                            <ProvidersSettings />
                          </PagePermissionGuard403>
                        }
                      />
                      <Route
                        path="providers/:providerId"
                        element={
                          <PagePermissionGuard403
                            permissions={[Permission.ProvidersRead]}
                          >
                            <ProvidersSettings />
                          </PagePermissionGuard403>
                        }
                      />
                      <Route
                        path="rag-providers"
                        element={
                          <PagePermissionGuard403
                            permissions={[Permission.RagProvidersRead]}
                          >
                            <RAGProvidersSettings />
                          </PagePermissionGuard403>
                        }
                      />
                      <Route
                        path="rag-providers/:providerId"
                        element={
                          <PagePermissionGuard403
                            permissions={[Permission.RagProvidersRead]}
                          >
                            <RAGProvidersSettings />
                          </PagePermissionGuard403>
                        }
                      />
                      <Route
                        path="repositories"
                        element={
                          <PagePermissionGuard403
                            permissions={[Permission.RepositoriesRead]}
                          >
                            <ModelRepositorySettings />
                          </PagePermissionGuard403>
                        }
                      />
                      <Route
                        path="hardware"
                        element={
                          <PagePermissionGuard403
                            permissions={[Permission.HardwareRead]}
                          >
                            <HardwareSettings />
                          </PagePermissionGuard403>
                        }
                      />
                      <Route
                        path="mcp-servers"
                        element={
                          <PagePermissionGuard403
                            permissions={[Permission.McpServersRead]}
                          >
                            <MCPServersSettings />
                          </PagePermissionGuard403>
                        }
                      />
                      <Route
                        path="admin-mcp-servers"
                        element={
                          <PagePermissionGuard403
                            permissions={[Permission.McpAdminServersRead]}
                          >
                            <MCPAdminPage />
                          </PagePermissionGuard403>
                        }
                      />
                      <Route
                        path="https-proxy"
                        element={
                          <PagePermissionGuard403
                            permissions={[Permission.ConfigProxyRead]}
                          >
                            <HttpsProxySettings />
                          </PagePermissionGuard403>
                        }
                      />
                      <Route
                        path="web-app"
                        element={
                          <PagePermissionGuard403
                            permissions={[Permission.ConfigNgrokRead]}
                          >
                            <NgrokSettings />
                          </PagePermissionGuard403>
                        }
                      />
                      <Route
                        path="api-proxy-server"
                        element={
                          <PagePermissionGuard403
                            permissions={[Permission.ApiProxyRead]}
                          >
                            <ApiProxyServerSettings />
                          </PagePermissionGuard403>
                        }
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
                        element={
                          <PagePermissionGuard403
                            permissions={[Permission.AssistantsAdminRead]}
                          >
                            <AdminAssistantsSettings />
                          </PagePermissionGuard403>
                        }
                      />
                      <Route
                        path="engines"
                        element={
                          <PagePermissionGuard403
                            permissions={[Permission.EnginesRead]}
                          >
                            <EnginesSettings />
                          </PagePermissionGuard403>
                        }
                      />
                      <Route
                        path="users"
                        element={
                          <PagePermissionGuard403
                            permissions={[Permission.UsersRead]}
                          >
                            <UsersSettings />
                          </PagePermissionGuard403>
                        }
                      />
                      <Route
                        path="user-groups"
                        element={
                          <PagePermissionGuard403
                            permissions={[Permission.GroupsRead]}
                          >
                            <UserGroupsSettings />
                          </PagePermissionGuard403>
                        }
                      />
                    </Route>
                  </Routes>
                </AppLayout>
              }
            />
          </Routes>
        </Router>
      </AuthGuard>
    </ThemeProvider>
  )
}

export default App
