// Auth store

import type { StoreApi, UseBoundStore } from 'zustand'
// Admin store
import { useShallow } from 'zustand/react/shallow'
import { useAdminStore } from './admin/admin.ts'
import { useUserAssistantsStore } from './assistants'
import { useAdminAssistantsStore } from './admin/assistants'
import { useAdminUsersStore } from './admin/users'
import { useAdminUserGroupsStore } from './admin/userGroups'
import { useAdminProxySettingsStore } from './admin/proxySettings'
import { useAuthStore } from './auth'
import { useChatStore } from './chat'
import { useChatHistoryStore } from './chatHistory'
import { useConversationsStore } from './conversations'
import { useHubStore } from './hub'
import { useLocalUploadStore } from './admin/localUpload.ts'
import { useModelDownloadStore } from './admin/modelDownload.ts'
import { useProjectsStore } from './projects'
import { useAdminProvidersStore } from './admin/providers.ts'
import { useUserProvidersStore } from './providers.ts'
import { useAdminRepositoriesStore } from './admin/repositories.ts'
import { useAdminRAGProvidersStore } from './admin/ragProviders.ts'
import { useAdminRAGRepositoriesStore } from './admin/ragRepositories.ts'
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
  useProjectDrawerStore,
  useViewDownloadModalStore,
} from './ui'

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
  clearChatError,
  createNewConversation,
  editChatMessage,
  loadConversationById,
  loadConversationMessageBranches,
  resetChatState,
  sendChatMessage,
  stopMessageStreaming,
  switchMessageBranch,
  useChatStore,
} from './chat'
// Chat History store
export {
  clearAllUserChatHistoryConversations,
  clearChatHistorySearchResults,
  clearChatHistoryStoreError,
  deleteChatHistoryConversationById,
  loadChatHistoryConversationsList,
  searchChatHistoryConversations,
  updateChatHistoryConversationTitleById,
  useChatHistoryStore,
} from './chatHistory'
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
// Projects store
export {
  clearProjectsStoreError,
  createNewProject,
  deleteExistingProject,
  loadAllUserProjects,
  loadProjectById,
  loadProjectWithDetails,
  resetProjectsStore,
  updateExistingProject,
  useProjectsStore,
} from './projects'
// Project Files functions (now in projects store)
export {
  cancelProjectFileUpload,
  clearFilesError,
  getProjectFiles,
  loadProjectFiles,
  uploadFilesToProject,
  removeProjectFileUploadProgress,
  getProjectFileUploadProgressById,
} from './projects'

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
  resetChatUI,
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
  setInputDisabled,
  setInputPlaceholder,
  setIsMobile,
  setMessageToolBoxVisible,
  setMobileOverlayOpen,
  setViewDownloadModalLoading,
  startEditingMessage,
  stopEditingMessage,
  updateEditingContent,
  useAddLocalModelDownloadDrawerStore,
  // Upload and Download modal stores
  useAddLocalModelUploadDrawerStore,
  useAddModelDrawerStore,
  useAddProviderDrawerStore,
  useAddRemoteModelDrawerStore,
  // Individual modal stores
  useAssistantDrawerStore,
  useChatUIStore,
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
} from './ui'

type ExtractState<T> = T extends UseBoundStore<StoreApi<infer State>>
  ? State
  : never

const createStoreProxy = <T extends UseBoundStore<StoreApi<any>>>(
  useStore: T,
): Readonly<ExtractState<T>> => {
  return new Proxy({} as Readonly<ExtractState<T>>, {
    get: (_, prop) => {
      return useStore(
        useShallow((state: ExtractState<T>) => (state as any)[prop]),
      )
    },
  })
}

export const Stores = {
  Auth: createStoreProxy(useAuthStore),
  Admin: createStoreProxy(useAdminStore),
  AdminUsers: createStoreProxy(useAdminUsersStore),
  AdminUserGroups: createStoreProxy(useAdminUserGroupsStore),
  AdminProxySettings: createStoreProxy(useAdminProxySettingsStore),
  Assistants: createStoreProxy(useUserAssistantsStore),
  AdminAssistants: createStoreProxy(useAdminAssistantsStore),
  Chat: createStoreProxy(useChatStore),
  ChatHistory: createStoreProxy(useChatHistoryStore),
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
  Settings: createStoreProxy(useUserSettingsStore),
  UI: {
    Chat: createStoreProxy(useChatUIStore),
    Layout: createStoreProxy(useLayoutUIStore),
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
