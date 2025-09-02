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
import { useRAGStore } from './rag'
import { useRAGInstanceDrawerStore } from './ui/ragDrawers'
import { useAdminProvidersStore } from './admin/providers.ts'
import { useUserProvidersStore } from './providers.ts'
import { useAdminRepositoriesStore } from './admin/repositories.ts'
import { useAdminRAGProvidersStore } from './admin/ragProviders.ts'
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
  useChatUIStore,
  useEditLocalModelDrawerStore,
  useEditProviderDrawerStore,
  useEditRemoteModelDrawerStore,
  useLayoutUIStore,
  usePathHistoryStore,
  useProjectDrawerStore,
  useViewDownloadModalStore,
} from './ui'
import {
  useAddRAGProviderDrawerStore,
  useEditRAGProviderDrawerStore,
  useAddSystemInstanceDrawerStore,
  useEditSystemInstanceDrawerStore,
} from './ui/ragProviderDrawers.ts'
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
  getGroupProviders,
  getGroupRagProviders,
  loadUserGroups,
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
  loadSystemUsers,
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
  loadHubModels,
  loadHubAssistants,
  refreshHubModels,
  refreshHubAssistants,
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
  clearRAGProvidersError,
  clearRAGInstanceError,
  createNewRAGProvider,
  createSystemRAGInstance,
  deleteRAGProvider,
  deleteSystemRAGInstance,
  disableSystemRAGInstance,
  enableSystemRAGInstance,
  findRAGInstanceById,
  findRAGProviderById,
  getCurrentRAGProvider,
  getInstancesForProvider,
  loadAllRAGProviders,
  loadInstancesForProvider,
  updateRAGProvider,
  updateSystemRAGInstance,
  useAdminRAGProvidersStore,
} from './admin/ragProviders.ts'

// RAG store (user-level)
export {
  loadAllUserRAGInstances,
  createRAGInstance,
  updateRAGInstanceInList,
  deleteRAGInstance,
  clearRAGStoreError,
  resetRAGStore,
  searchRAGInstances,
  toggleSystemInstances,
  useRAGStore,
} from './rag'

// RAG Instance store
export {
  createRAGInstanceStore,
  useRAGInstanceStore,
} from './ragInstance'

// RAG UI stores
export {
  openRAGInstanceDrawer,
  closeRAGInstanceDrawer,
  setRAGInstanceDrawerLoading,
  useRAGInstanceDrawerStore,
} from './ui/ragDrawers'

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
  updateUserSetting,
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
  // Chat Input UI store
  createChatInputUIStore,
  // Chat UI store
  useChatUIStore,
  toggleShowTime,
  setShowTime,
} from './ui'

// RAG Provider UI stores
export {
  openAddRAGProviderDrawer,
  closeAddRAGProviderDrawer,
  setAddRAGProviderDrawerLoading,
  openEditRAGProviderDrawer,
  closeEditRAGProviderDrawer,
  setEditRAGProviderDrawerLoading,
  openAddSystemInstanceDrawer,
  closeAddSystemInstanceDrawer,
  setAddSystemInstanceDrawerLoading,
  openEditSystemInstanceDrawer,
  closeEditSystemInstanceDrawer,
  setEditSystemInstanceDrawerLoading,
} from './ui/ragProviderDrawers.ts'

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
  RAG: createStoreProxy(useRAGStore),
  Providers: createStoreProxy(useUserProvidersStore),
  AdminProviders: createStoreProxy(useAdminProvidersStore),
  AdminModels: createStoreProxy(useAdminProvidersStore), // Legacy compatibility
  AdminRepositories: createStoreProxy(useAdminRepositoriesStore),
  AdminRAGProviders: createStoreProxy(useAdminRAGProvidersStore),
  AdminApiProxyServer: createStoreProxy(useApiProxyServerStore),
  AdminApiProxyLogMonitor: createStoreProxy(useApiProxyLogMonitorStore),
  AdminEngines: createStoreProxy(useEngineStore),
  Settings: createStoreProxy(useUserSettingsStore),
  UI: {
    Layout: createStoreProxy(useLayoutUIStore),
    PathHistory: createStoreProxy(usePathHistoryStore),
    ChatUI: createStoreProxy(useChatUIStore),
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
    RAGInstanceDrawer: createStoreProxy(useRAGInstanceDrawerStore),
    // RAG drawer stores
    AddRAGProviderDrawer: createStoreProxy(useAddRAGProviderDrawerStore),
    EditRAGProviderDrawer: createStoreProxy(useEditRAGProviderDrawerStore),
    AddSystemInstanceDrawer: createStoreProxy(useAddSystemInstanceDrawerStore),
    EditSystemInstanceDrawer: createStoreProxy(
      useEditSystemInstanceDrawerStore,
    ),
  },
}
