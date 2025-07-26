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
import { useHubStore } from "./hub";
import { useLocalUploadStore } from "./localUpload";
import { useModelDownloadStore } from "./modelDownload";
import { useProjectsStore } from "./projects";
import { useProvidersStore } from "./providers";
import { useRepositoriesStore } from "./repositories";
import { useUserSettingsStore } from "./settings";
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
} from "./assistants";
export {
  authenticateUser,
  clearAuthenticationError,
  logoutUser,
  registerNewUser,
  setupInitialAdminUser,
  useAuthStore,
} from "./auth";
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
} from "./chat";
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
} from "./chatHistory";
// Conversations store
export {
  addNewConversationToList,
  clearConversationsStoreError,
  loadAllRecentConversations,
  removeConversationFromList,
  setConversationsListLoading,
  updateExistingConversation,
  useConversationsStore,
} from "./conversations";
// Local Upload store
export {
  cancelLocalUpload,
  clearLocalUploadError,
  hideUploadProgress,
  showUploadProgress,
  uploadLocalModel,
  useLocalUploadStore,
} from "./localUpload";
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
} from "./modelDownload";
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
} from "./hub";
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
} from "./projects";
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
} from "./providers";
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
} from "./repositories";
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
} from "./settings";

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
} from "./ui";

type ExtractState<T> =
  T extends UseBoundStore<StoreApi<infer State>> ? State : never;

const createStoreProxy = <T extends UseBoundStore<StoreApi<any>>>(
  useStore: T,
): Readonly<ExtractState<T>> => {
  return new Proxy({} as Readonly<ExtractState<T>>, {
    get: (_, prop) => {
      return useStore(
        useShallow((state: ExtractState<T>) => (state as any)[prop]),
      );
    },
  });
};

export const Stores = {
  Auth: createStoreProxy(useAuthStore),
  Admin: createStoreProxy(useAdminStore),
  Assistants: createStoreProxy(useAssistantsStore),
  Chat: createStoreProxy(useChatStore),
  ChatHistory: createStoreProxy(useChatHistoryStore),
  Conversations: createStoreProxy(useConversationsStore),
  Hub: createStoreProxy(useHubStore),
  LocalUpload: createStoreProxy(useLocalUploadStore),
  ModelDownload: createStoreProxy(useModelDownloadStore),
  Projects: createStoreProxy(useProjectsStore),
  Providers: createStoreProxy(useProvidersStore),
  Repositories: createStoreProxy(useRepositoriesStore),
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
  },
};
