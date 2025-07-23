// Auth store

import type { StoreApi, UseBoundStore } from "zustand";
// Admin store
import { useShallow } from "zustand/react/shallow";
import { useAdminStore } from "./admin";
import { useAssistantsStore } from "./assistants";
import { useAuthStore } from "./auth";
import { useChatStore } from "./chat";
import { useChatHistoryStore } from "./chatHistory";
import { useConversationsStore } from "./conversations";
import { useModelDownloadStore } from "./modelDownload";
import { useProjectsStore } from "./projects";
import { useProvidersStore } from "./providers";
import { useRepositoriesStore } from "./repositories";
import { useUserSettingsStore } from "./settings";
import { useChatUIStore, useLayoutUIStore, useModalsUIStore } from "./ui";

export {
  assignUserToUserGroup,
  clearSystemAdminError,
  createNewUserGroup,
  createSystemAdminAssistant,
  deleteSystemAdminAssistant,
  deleteUserGroup,
  loadAllSystemUsers,
  loadAllUserGroups,
  loadSystemAdminAssistants,
  loadSystemProxySettings,
  loadSystemUserRegistrationSettings,
  loadUserGroupMembers,
  removeUserFromUserGroup,
  resetSystemUserPassword,
  toggleSystemUserActiveStatus,
  updateSystemAdminAssistant,
  updateSystemDefaultLanguage,
  updateSystemProxySettings,
  updateSystemUser,
  updateSystemUserRegistrationSettings,
  updateUserGroup,
  useAdminStore,
} from "./admin";
// Assistants store
export {
  clearAssistantsStoreError,
  createAdministratorAssistant,
  createUserAssistant,
  deleteAdministratorAssistant,
  deleteUserAssistant,
  loadAdministratorAssistants,
  loadUserAssistants,
  updateAdministratorAssistant,
  updateUserAssistant,
  useAssistantsStore,
} from './assistants'
export {
  authenticateUser,
  checkApplicationInitializationStatus,
  clearAuthenticationError,
  fetchCurrentUserProfile,
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
// Model Download store
export {
  cancelModelDownload,
  clearAllModelDownloads,
  clearModelDownload,
  closeDownloadModal,
  downloadModelFromRepository,
  findDownloadById,
  getAllActiveDownloads,
  openAddModelModal as openDownloadModelModal,
  openViewDownloadModal,
  useModelDownloadStore,
} from './modelDownload'
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
  uploadDocumentToProject,
  useProjectsStore,
} from './projects'
// Providers store
export {
  addNewModel,
  addNewModelToProvider,
  cancelModelUpload,
  clearProvidersError,
  cloneExistingProvider,
  createNewModelProvider,
  deleteExistingModel,
  deleteModelProvider,
  disableModelFromUse,
  enableModelForUse,
  findModelById,
  findProviderById,
  loadAllModelProviders,
  loadModels,
  loadModelsForProvider,
  startModelExecution,
  stopModelExecution,
  updateExistingModel,
  updateModelProvider,
  uploadModelFilesAndCreateModel,
  useProvidersStore,
} from './providers'
// Repositories store
export {
  clearRepositoriesStoreError,
  createNewModelRepository,
  deleteModelRepository,
  findRepositoryById,
  loadAllModelRepositories,
  testModelRepositoryConnection,
  updateModelRepository,
  useRepositoriesStore,
} from './repositories'
// Settings store
export {
  deleteUserSetting,
  getResolvedAppearanceTheme,
  getUILeftPanelCollapsed,
  getUILeftPanelWidth,
  getUserAppearanceComponentSize,
  getUserAppearanceLanguage,
  getUserAppearanceTheme,
  getUserSetting,
  initializeUserSettingsOnStartup,
  loadGlobalDefaultLanguage,
  loadUserSettings,
  resetAllUserSettings,
  saveUserSetting,
  setUILeftPanelCollapsed,
  setUILeftPanelWidth,
  updateUserSetting,
  useAppearanceSettings,
  useUISettings,
  useUserSettingsStore,
} from './settings'

// UI stores with all actions
export {
  closeAddModelModal,
  closeAddProviderModal,
  closeAssistantModal,
  closeEditModelModal,
  closeEditProviderModal,
  closeMobileOverlay,
  openAddModelModal,
  openAddProviderModal,
  openAssistantModal,
  openEditModelModal,
  openEditProviderModal,
  resetChatUI,
  resetModals,
  setAddModelModalLoading,
  setAddProviderModalLoading,
  setAssistantModalLoading,
  setEditModelModalLoading,
  setEditProviderModalLoading,
  setInputDisabled,
  setInputPlaceholder,
  setIsMobile,
  setMessageToolBoxVisible,
  setMobileOverlayOpen,
  startEditingMessage,
  stopEditingMessage,
  updateEditingContent,
  useChatUIStore,
  useLayoutUIStore,
  useModalsUIStore,
} from './ui'

type ExtractState<T> = T extends UseBoundStore<StoreApi<infer State>>
  ? State
  : never

const createStoreProxy = <T extends UseBoundStore<StoreApi<any>>>(
  useStore: T,
): ExtractState<T> => {
  return new Proxy({} as ExtractState<T>, {
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
  Assistants: createStoreProxy(useAssistantsStore),
  Chat: createStoreProxy(useChatStore),
  ChatHistory: createStoreProxy(useChatHistoryStore),
  Conversations: createStoreProxy(useConversationsStore),
  ModelDownload: createStoreProxy(useModelDownloadStore),
  Projects: createStoreProxy(useProjectsStore),
  Providers: createStoreProxy(useProvidersStore),
  Repositories: createStoreProxy(useRepositoriesStore),
  Settings: createStoreProxy(useUserSettingsStore),
  UI: {
    Modals: createStoreProxy(useModalsUIStore),
    Chat: createStoreProxy(useChatUIStore),
    Layout: createStoreProxy(useLayoutUIStore),
  },
}
