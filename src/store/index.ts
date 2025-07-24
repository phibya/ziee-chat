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
import { useLocalUploadStore } from "./localUpload";
import { useModelDownloadStore } from "./modelDownload";
import { useProjectsStore } from "./projects";
import { useProvidersStore } from "./providers";
import { useRepositoriesStore } from "./repositories";
import { useUserSettingsStore } from "./settings";
import {
  useAddLocalModelDownloadModalStore,
  useAddLocalModelUploadModalStore,
  useAddModelModalStore,
  useAddProviderModalStore,
  useAddRemoteModelModalStore,
  useAssistantModalStore,
  useChatUIStore,
  useEditLocalModelModalStore,
  useEditProviderModalStore,
  useEditRemoteModelModalStore,
  useLayoutUIStore,
  useViewDownloadModalStore,
} from "./ui";

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
  hideUploadProgress,
  showUploadProgress,
  uploadLocalModel,
  useLocalUploadStore,
} from './localUpload'
// Model Download store
export {
  cancelModelDownload,
  clearAllModelDownloads,
  clearModelDownload,
  downloadModelFromRepository,
  findDownloadById,
  getAllActiveDownloads,
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
  closeAddLocalModelDownloadModal,
  closeAddLocalModelUploadModal,
  closeAddModelModal,
  closeAddProviderModal,
  closeAddRemoteModelModal,
  closeAssistantModal,
  closeEditLocalModelModal,
  closeEditProviderModal,
  closeEditRemoteModelModal,
  closeMobileOverlay,
  closeViewDownloadModal,
  openAddLocalModelDownloadModal,
  openAddLocalModelUploadModal,
  openAddModelModal,
  openAddProviderModal,
  openAddRemoteModelModal,
  openAssistantModal,
  openEditLocalModelModal,
  openEditProviderModal,
  openEditRemoteModelModal,
  openViewDownloadModal,
  resetChatUI,
  setAddLocalModelDownloadModalLoading,
  setAddLocalModelUploadModalLoading,
  setAddModelModalLoading,
  setAddProviderModalLoading,
  setAddRemoteModelModalLoading,
  setAssistantModalLoading,
  setEditLocalModelModalLoading,
  setEditProviderModalLoading,
  setEditRemoteModelModalLoading,
  setInputDisabled,
  setInputPlaceholder,
  setIsMobile,
  setMessageToolBoxVisible,
  setMobileOverlayOpen,
  setViewDownloadModalLoading,
  startEditingMessage,
  stopEditingMessage,
  updateEditingContent,
  useAddLocalModelDownloadModalStore,
  // Upload and Download modal stores
  useAddLocalModelUploadModalStore,
  useAddModelModalStore,
  useAddProviderModalStore,
  useAddRemoteModelModalStore,
  // Individual modal stores
  useAssistantModalStore,
  useChatUIStore,
  useEditLocalModelModalStore,
  useEditProviderModalStore,
  useEditRemoteModelModalStore,
  useLayoutUIStore,
  useViewDownloadModalStore,
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
  Assistants: createStoreProxy(useAssistantsStore),
  Chat: createStoreProxy(useChatStore),
  ChatHistory: createStoreProxy(useChatHistoryStore),
  Conversations: createStoreProxy(useConversationsStore),
  LocalUpload: createStoreProxy(useLocalUploadStore),
  ModelDownload: createStoreProxy(useModelDownloadStore),
  Projects: createStoreProxy(useProjectsStore),
  Providers: createStoreProxy(useProvidersStore),
  Repositories: createStoreProxy(useRepositoriesStore),
  Settings: createStoreProxy(useUserSettingsStore),
  UI: {
    Chat: createStoreProxy(useChatUIStore),
    Layout: createStoreProxy(useLayoutUIStore),
    // Individual modal stores
    AssistantModal: createStoreProxy(useAssistantModalStore),
    AddProviderModal: createStoreProxy(useAddProviderModalStore),
    EditProviderModal: createStoreProxy(useEditProviderModalStore),
    AddModelModal: createStoreProxy(useAddModelModalStore),
    ViewDownloadModal: createStoreProxy(useViewDownloadModalStore),
    AddRemoteModelModal: createStoreProxy(useAddRemoteModelModalStore),
    EditLocalModelModal: createStoreProxy(useEditLocalModelModalStore),
    EditRemoteModelModal: createStoreProxy(useEditRemoteModelModalStore),
    AddLocalModelUploadModal: createStoreProxy(
      useAddLocalModelUploadModalStore,
    ),
    AddLocalModelDownloadModal: createStoreProxy(
      useAddLocalModelDownloadModalStore,
    ),
  },
}
