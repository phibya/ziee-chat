import { useAdminStore } from './admin/admin.ts'
import { useUserAssistantsStore } from './assistants'
import { useAdminAssistantsStore } from './admin/assistants'
import { useAdminUsersStore } from './admin/users'
import { useAdminUserGroupsStore } from './admin/userGroups'
import { useAdminProxySettingsStore } from './admin/proxySettings'
import { useAdminNgrokSettingsStore } from './admin/ngrokSettings'
import { useHardwareStore } from './admin/hardware'
import { useAuthStore } from './auth'
import { useConversationsStore } from './conversations'
import { useHubStore } from './hub'
import { useLocalUploadStore } from './admin/localUpload.ts'
import { useModelDownloadStore } from './admin/modelDownload.ts'
import { useProjectsStore } from './projects'
// import { useProjectStore } from './project' // Imported via export below
import { useAdminProvidersStore } from './admin/providers.ts'
import { useUserProvidersStore } from './providers.ts'
import { useAdminRepositoriesStore } from './admin/repositories.ts'
import { useAdminRAGProvidersStore } from './admin/ragProviders.ts'
import { useAdminRAGRepositoriesStore } from './admin/ragRepositories.ts'
import { useApiProxyServerStore } from './admin/apiProxyServer.ts'
import { useApiProxyLogMonitorStore } from './admin/apiProxyLogMonitor.ts'
import { useEngineStore } from './engine'
import { useUserSettingsStore } from './settings'
import {
  useAddLocalModelDownloadDrawerStore,
  useAddLocalModelUploadDrawerStore,
  useAddModelDrawerStore,
  useAddProviderDrawerStore,
  useAddRemoteModelDrawerStore,
  useAssistantDrawerStore,
  useEditLocalModelDrawerStore,
  useEditProviderDrawerStore,
  useEditRemoteModelDrawerStore,
  useLayoutUIStore,
  usePathHistoryStore,
  useProjectDrawerStore,
  useViewDownloadModalStore,
} from './ui'
import { createStoreProxy } from '../utils/createStoreProxy.ts'

export {
  clearSystemAdminError,
  updateSystemDefaultLanguage,
  useAdminStore,
} from './admin/admin.ts'
// Admin User Groups store
export {
  assignUserToUserGroup,
  clearAdminUserGroupsStoreError,
  createNewUserGroup,
  deleteUserGroup,
  loadAllUserGroups,
  loadUserGroupMembers,
  removeUserFromUserGroup,
  updateUserGroup,
  useAdminUserGroupsStore,
} from './admin/userGroups'
// Admin Proxy Settings store
export {
  clearAdminProxySettingsStoreError,
  loadSystemProxySettings,
  updateSystemProxySettings,
  useAdminProxySettingsStore,
} from './admin/proxySettings'
// Admin Ngrok Settings store
export {
  loadNgrokSettings,
  updateNgrokSettings,
  updateAccountPassword,
  startNgrokTunnel,
  stopNgrokTunnel,
  refreshNgrokStatus,
  useAdminNgrokSettingsStore,
} from './admin/ngrokSettings'
// Admin Hardware store
export {
  clearHardwareError,
  loadHardwareInfo,
  subscribeToHardwareUsage,
  disconnectHardwareUsage,
  useHardwareStore,
} from './admin/hardware'
// Admin Users store
export {
  clearAdminUsersStoreError,
  loadAllSystemUsers,
  loadSystemUserRegistrationSettings,
  resetSystemUserPassword,
  toggleSystemUserActiveStatus,
  updateSystemUser,
  updateSystemUserRegistrationSettings,
  useAdminUsersStore,
} from './admin/users'
// User Assistants store
export {
  clearUserAssistantsStoreError,
  createUserAssistant,
  deleteUserAssistant,
  loadUserAssistants,
  updateUserAssistant,
  useUserAssistantsStore,
  // Legacy compatibility
  useAssistantsStore,
  clearAssistantsStoreError,
} from './assistants'
// Admin Assistants store
export {
  clearAdminAssistantsStoreError,
  createSystemAdminAssistant,
  deleteSystemAdminAssistant,
  loadAdministratorAssistants,
  updateSystemAdminAssistant,
  useAdminAssistantsStore,
  // Legacy compatibility
  loadSystemAdminAssistants,
} from './admin/assistants'
export {
  authenticateUser,
  clearAuthenticationError,
  logoutUser,
  registerNewUser,
  setupInitialAdminUser,
  useAuthStore,
} from './auth'
// Chat store
export {
  createNewConversation,
  useChatStore,
} from './chat'
// Chat History store
export { useChatHistoryStore } from './chatHistory'
// Conversations store
export {
  addNewConversationToList,
  clearConversationsStoreError,
  loadAllRecentConversations,
  removeConversationFromList,
  setConversationsListLoading,
  updateExistingConversation,
  useConversationsStore,
} from './conversations'

// Local Upload store
export {
  cancelLocalUpload,
  clearLocalUploadError,
  hideUploadProgress as hideLocalUploadProgress,
  showUploadProgress as showLocalUploadProgress,
  uploadLocalModel,
  useLocalUploadStore,
} from './admin/localUpload.ts'
// Model Download store
export {
  cancelModelDownload,
  clearAllModelDownloads,
  clearModelDownload,
  deleteModelDownload,
  downloadModelFromRepository,
  findDownloadById,
  getAllActiveDownloads,
  useModelDownloadStore,
} from './admin/modelDownload.ts'
// Hub store
export {
  initializeHub,
  refreshHub,
  getHubVersion,
  searchModels,
  searchAssistants,
  getModelsByCategory,
  getAssistantsByCategory,
  useHubStore,
} from './hub'
// Projects store (project list)
export {
  clearProjectsStoreError,
  createNewProject,
  deleteExistingProject,
  loadAllUserProjects,
  resetProjectsStore,
  updateProjectInList,
  useProjectsStore,
} from './projects'
// Project store (individual project)
export {
  createProjectStore,
  useProjectStore,
} from './project'

export {
  getFile,
  getFileContent,
  getFileThumbnail,
  getFileThumbnails,
  uploadFile,
  deleteFile,
} from './files'

// User Providers store
export {
  clearUserProvidersError,
  findUserModelById,
  findUserProviderById,
  loadUserModelsForProvider,
  loadUserProviders,
  loadUserProvidersWithAllModels,
  useUserProvidersStore,
} from './providers.ts'
// Admin Providers store
export {
  clearProvidersError,
  createNewModelProvider,
  deleteModelProvider,
  findProviderById,
  loadAllModelProviders,
  updateModelProvider,
  useAdminProvidersStore,
  // Models functions (now in providers store)
  addNewModel,
  addNewModelToProvider,
  clearModelError,
  deleteExistingModel,
  disableModelFromUse,
  enableModelForUse,
  findModelById,
  loadModelsForProvider,
  startModelExecution,
  stopModelExecution,
  updateExistingModel,
  getModelsForProvider,
  getCurrentProvider,
} from './admin/providers.ts'
// Admin Repositories store
export {
  adminRepositoryHasCredentials,
  clearAdminRepositoriesStoreError,
  createNewAdminModelRepository,
  deleteAdminModelRepository,
  findAdminRepositoryById,
  loadAllAdminModelRepositories,
  testAdminModelRepositoryConnection,
  updateAdminModelRepository,
  useAdminRepositoriesStore,
} from './admin/repositories.ts'
// Admin RAG Providers store
export {
  addNewDatabaseToRAGProvider,
  clearRAGProvidersError,
  cloneExistingRAGProvider,
  createNewRAGProvider,
  deleteExistingRAGDatabase,
  deleteRAGProvider,
  disableRAGDatabase,
  enableRAGDatabase,
  findRAGDatabaseById,
  findRAGProviderById,
  loadAllRAGProviders,
  loadDatabasesForRAGProvider,
  startRAGDatabase,
  stopRAGDatabase,
  updateExistingRAGDatabase,
  updateRAGProvider,
  useAdminRAGProvidersStore,
} from './admin/ragProviders.ts'
// Admin RAG Repositories store
export {
  clearRAGRepositoriesError,
  createNewRAGRepository,
  deleteRAGRepository,
  downloadRAGDatabaseFromRepository,
  findRAGRepositoryById,
  loadAllRAGRepositories,
  loadAvailableDatabasesFromRepository,
  searchAvailableRAGDatabases,
  searchRAGRepositories,
  testRAGRepositoryConnection,
  updateRAGRepository,
  useAdminRAGRepositoriesStore,
} from './admin/ragRepositories.ts'
// Admin API Proxy Server store
export {
  loadApiProxyServerConfig,
  updateApiProxyServerConfig,
  loadApiProxyServerStatus,
  startApiProxyServer,
  stopApiProxyServer,
  loadApiProxyServerModels,
  addModelToApiProxyServer,
  updateApiProxyServerModel,
  removeModelFromApiProxyServer,
  loadApiProxyServerTrustedHosts,
  addTrustedHostToApiProxyServer,
  updateApiProxyServerTrustedHost,
  removeTrustedHostFromApiProxyServer,
  initializeApiProxyServerData,
  refreshApiProxyServerData,
  useApiProxyServerStore,
} from './admin/apiProxyServer.ts'
// Admin API Proxy Log Monitor store
export {
  connectToApiProxyLogs,
  disconnectFromApiProxyLogs,
  clearLogBuffer,
  setAutoScroll,
  downloadLogs,
  useApiProxyLogMonitorStore,
} from './admin/apiProxyLogMonitor.ts'
// Engine store
export {
  initializeEngines,
  getEngineByType,
  getAvailableEngines,
  searchEngines,
  useEngineStore,
} from './engine'
// Settings store
export {
  deleteUserSetting,
  getUserSetting,
  initializeUserSettings,
  loadGlobalDefaultLanguage,
  loadUserSettings,
  resetAllUserSettings,
  saveUserSetting,
  setUILeftPanelCollapsed,
  setUILeftPanelWidth,
  updateUserSetting,
  useUILeftPanelCollapsed,
  useUILeftPanelWidth,
  useUserAppearanceLanguage,
  useUserAppearanceTheme,
  useUserSettings,
  useUserSettingsStore,
} from './settings'

// UI stores with all actions
export {
  closeAddLocalModelDownloadDrawer,
  closeAddLocalModelUploadDrawer,
  closeAddModelDrawer,
  closeAddProviderDrawer,
  closeAddRemoteModelDrawer,
  closeAssistantDrawer,
  closeEditLocalModelDrawer,
  closeEditProviderDrawer,
  closeEditRemoteModelDrawer,
  closeMobileOverlay,
  closeProjectDrawer,
  closeViewDownloadModal,
  // RAG drawer actions
  closeAddRAGProviderDrawer,
  closeAddRAGDatabaseDrawer,
  closeAddRAGDatabaseDownloadDrawer,
  closeEditRAGDatabaseDrawer,
  openAddRAGProviderDrawer,
  openAddRAGDatabaseDrawer,
  openAddRAGDatabaseDownloadDrawer,
  openEditRAGDatabaseDrawer,
  setAddRAGProviderDrawerLoading,
  setAddRAGDatabaseDrawerLoading,
  setAddRAGDatabaseDownloadDrawerLoading,
  setEditRAGDatabaseDrawerLoading,
  openAddLocalModelDownloadDrawer,
  openAddLocalModelUploadDrawer,
  openAddModelDrawer,
  openAddProviderDrawer,
  openAddRemoteModelDrawer,
  openAssistantDrawer,
  openEditLocalModelDrawer,
  openEditProviderDrawer,
  openEditRemoteModelDrawer,
  openProjectDrawer,
  openViewDownloadModal,
  setAddLocalModelDownloadDrawerLoading,
  setAddLocalModelUploadDrawerLoading,
  setAddModelDrawerLoading,
  setAddProviderDrawerLoading,
  setAddRemoteModelDrawerLoading,
  setAssistantDrawerLoading,
  setEditLocalModelDrawerLoading,
  setEditProviderDrawerLoading,
  setEditRemoteModelDrawerLoading,
  setProjectDrawerLoading,
  setIsMobile,
  setMobileOverlayOpen,
  setSidebarCollapsed,
  toggleSidebar,
  setMainContentWidth,
  setViewDownloadModalLoading,
  useAddLocalModelDownloadDrawerStore,
  // Upload and Download modal stores
  useAddLocalModelUploadDrawerStore,
  useAddModelDrawerStore,
  useAddProviderDrawerStore,
  useAddRemoteModelDrawerStore,
  // Individual modal stores
  useAssistantDrawerStore,
  useEditLocalModelDrawerStore,
  useEditProviderDrawerStore,
  useEditRemoteModelDrawerStore,
  useLayoutUIStore,
  useViewDownloadModalStore,
  // RAG drawer stores
  useAddRAGProviderDrawerStore,
  useAddRAGDatabaseDrawerStore,
  useAddRAGDatabaseDownloadDrawerStore,
  useEditRAGDatabaseDrawerStore,
  // Chat Input UI store
  createChatInputUIStore,
} from './ui'

export const Stores = {
  Auth: createStoreProxy(useAuthStore),
  Admin: createStoreProxy(useAdminStore),
  AdminUsers: createStoreProxy(useAdminUsersStore),
  AdminUserGroups: createStoreProxy(useAdminUserGroupsStore),
  AdminProxySettings: createStoreProxy(useAdminProxySettingsStore),
  AdminNgrokSettings: createStoreProxy(useAdminNgrokSettingsStore),
  AdminHardware: createStoreProxy(useHardwareStore),
  Assistants: createStoreProxy(useUserAssistantsStore),
  AdminAssistants: createStoreProxy(useAdminAssistantsStore),
  Conversations: createStoreProxy(useConversationsStore),
  Hub: createStoreProxy(useHubStore),
  LocalUpload: createStoreProxy(useLocalUploadStore),
  ModelDownload: createStoreProxy(useModelDownloadStore),
  Projects: createStoreProxy(useProjectsStore),
  Providers: createStoreProxy(useUserProvidersStore),
  AdminProviders: createStoreProxy(useAdminProvidersStore),
  AdminModels: createStoreProxy(useAdminProvidersStore), // Legacy compatibility
  AdminRepositories: createStoreProxy(useAdminRepositoriesStore),
  AdminRAGProviders: createStoreProxy(useAdminRAGProvidersStore),
  AdminRAGRepositories: createStoreProxy(useAdminRAGRepositoriesStore),
  AdminApiProxyServer: createStoreProxy(useApiProxyServerStore),
  AdminApiProxyLogMonitor: createStoreProxy(useApiProxyLogMonitorStore),
  AdminEngines: createStoreProxy(useEngineStore),
  Settings: createStoreProxy(useUserSettingsStore),
  UI: {
    Layout: createStoreProxy(useLayoutUIStore),
    PathHistory: createStoreProxy(usePathHistoryStore),
    // Individual drawer stores
    AssistantDrawer: createStoreProxy(useAssistantDrawerStore),
    AddProviderDrawer: createStoreProxy(useAddProviderDrawerStore),
    EditProviderDrawer: createStoreProxy(useEditProviderDrawerStore),
    AddModelDrawer: createStoreProxy(useAddModelDrawerStore),
    ViewDownloadDrawer: createStoreProxy(useViewDownloadModalStore),
    AddRemoteModelDrawer: createStoreProxy(useAddRemoteModelDrawerStore),
    EditLocalModelDrawer: createStoreProxy(useEditLocalModelDrawerStore),
    EditRemoteModelDrawer: createStoreProxy(useEditRemoteModelDrawerStore),
    AddLocalModelUploadDrawer: createStoreProxy(
      useAddLocalModelUploadDrawerStore,
    ),
    AddLocalModelDownloadDrawer: createStoreProxy(
      useAddLocalModelDownloadDrawerStore,
    ),
    ProjectDrawer: createStoreProxy(useProjectDrawerStore),
    // RAG drawer stores - temporarily commented out for debugging
    // AddRAGProviderDrawer: createStoreProxy(useAddRAGProviderDrawerStore),
    // AddRAGDatabaseDrawer: createStoreProxy(useAddRAGDatabaseDrawerStore),
    // AddRAGDatabaseDownloadDrawer: createStoreProxy(useAddRAGDatabaseDownloadDrawerStore),
    // EditRAGDatabaseDrawer: createStoreProxy(useEditRAGDatabaseDrawerStore),
  },
}
