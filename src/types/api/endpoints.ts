/**
 * API endpoint type definitions
 * Centralized location for all API request/response types
 */

import {
  Assistant,
  AssistantListResponse,
  CreateAssistantRequest,
  UpdateAssistantRequest,
} from './assistant'
import { AuthResponse, InitResponse, LoginRequest } from './auth'
import {
  Conversation,
  ConversationListResponse,
  CreateConversationRequest,
  Message,
  MessageBranch,
  SendMessageRequest,
  SwitchBranchRequest,
  UpdateConversationRequest,
} from './chat'
import {
  NgrokSettingsResponse,
  NgrokStatusResponse,
  ProxySettingsResponse,
  TestProxyConnectionRequest,
  TestProxyConnectionResponse,
  UpdateAccountPasswordRequest,
  UpdateNgrokSettingsRequest,
  UpdateProxySettingsRequest,
  UpdateUserRegistrationRequest,
  UserRegistrationStatusResponse,
} from './config.ts'
import {
  DefaultLanguageResponse,
  UpdateDefaultLanguageRequest,
} from './globalConfig'
import { HardwareInfoResponse } from './hardware'
import {
  File,
  FileListParams,
  FileListResponse,
  UploadFileResponse,
  DownloadTokenResponse,
} from './files'
import { HubDataResponse, HubVersionResponse } from './hub'
import {
  AddModelToProviderRequest,
  Model,
  ModelCapabilities,
  ModelParameters,
  MistralRsSettings,
  LlamaCppSettings,
  UpdateModelRequest,
} from './model'
import {
  DownloadFromRepositoryRequest,
  DownloadInstance,
  DownloadInstanceListResponse,
} from './modelDownloads.ts'
import {
  CreateProjectRequest,
  Project,
  ProjectDetailResponse,
  ProjectListParams,
  ProjectListResponse,
  UpdateProjectRequest,
} from './projects'
import {
  AvailableDevicesResponse,
  CreateProviderRequest,
  Provider,
  ProviderListResponse,
  UpdateProviderRequest,
} from './provider'
import {
  CreateRepositoryRequest,
  Repository,
  RepositoryListResponse,
  TestRepositoryConnectionRequest,
  TestRepositoryConnectionResponse,
  UpdateRepositoryRequest,
} from './repository'
import {
  AssignUserToGroupRequest,
  CreateUserGroupRequest,
  CreateUserRequest,
  ResetPasswordRequest,
  UpdateUserGroupRequest,
  UpdateUserRequest,
  User,
  UserGroup,
  UserListResponse,
} from './user'
import { UserGroupListResponse } from './userGroup.ts'
import {
  UserSetting,
  UserSettingRequest,
  UserSettingsResponse,
} from './userSettings'
import {
  CreateRAGProviderRequest,
  CreateRAGDatabaseRequest,
  RAGProvider,
  RAGProviderListResponse,
  RAGDatabase,
  UpdateRAGProviderRequest,
  UpdateRAGDatabaseRequest,
} from './ragProvider'
import {
  CreateRAGRepositoryRequest,
  DownloadRAGDatabaseFromRepositoryRequest,
  RAGRepository,
  RAGRepositoryConnectionTestResponse,
  RAGRepositoryListResponse,
  UpdateRAGRepositoryRequest,
} from './ragRepository'
import { EngineListResponse } from './engine'

// API endpoint definitions
export const ApiEndpoints = {
  // ===========================
  // CORE APPLICATION ENDPOINTS
  // ===========================

  // App system
  'App.getHttpPort': 'GET /get_http_port',

  // Authentication
  'Auth.init': 'GET /api/auth/init',
  'Auth.setup': 'POST /api/auth/setup',
  'Auth.login': 'POST /api/auth/login',
  'Auth.logout': 'POST /api/auth/logout',
  'Auth.register': 'POST /api/auth/register',
  'Auth.me': 'GET /api/auth/me',

  // Public configuration
  'Config.getUserRegistrationStatus': 'GET /api/config/user-registration',
  'Config.getDefaultLanguage': 'GET /api/config/default-language',

  // User settings
  'UserSettings.getAll': 'GET /api/user/settings',
  'UserSettings.get': 'GET /api/user/settings/{key}',
  'UserSettings.set': 'POST /api/user/settings',
  'UserSettings.delete': 'DELETE /api/user/settings/{key}',
  'UserSettings.deleteAll': 'DELETE /api/user/settings/all',

  // Utilities
  'Utils.testProxy': 'POST /api/utils/test-proxy',
  'User.greet': 'POST /api/user/greet',

  // ===========================
  // USER FEATURES
  // ===========================

  // User Assistants
  'Assistant.list': 'GET /api/assistants',
  'Assistant.create': 'POST /api/assistants',
  'Assistant.get': 'GET /api/assistants/{assistant_id}',
  'Assistant.update': 'PUT /api/assistants/{assistant_id}',
  'Assistant.delete': 'DELETE /api/assistants/{assistant_id}',
  'Assistant.getDefault': 'GET /api/assistants/default',

  // Chat Management
  'Chat.listConversations': 'GET /api/chat/conversations',
  'Chat.createConversation': 'POST /api/chat/conversations',
  'Chat.getConversation': 'GET /api/chat/conversations/{conversation_id}',
  'Chat.updateConversation': 'PUT /api/chat/conversations/{conversation_id}',
  'Chat.deleteConversation': 'DELETE /api/chat/conversations/{conversation_id}',
  'Chat.sendMessageStream': 'POST /api/chat/messages/stream',
  'Chat.editMessageStream': 'PUT /api/chat/messages/{message_id}/stream',
  'Chat.getMessageBranches': 'GET /api/chat/messages/{message_id}/branches',
  'Chat.getConversationMessages':
    'GET /api/chat/conversations/{conversation_id}/messages/{branch_id}',
  'Chat.switchConversationBranch':
    'PUT /api/chat/conversations/{conversation_id}/branch/switch',
  'Chat.searchConversations': 'GET /api/chat/conversations/search',

  // Project Management
  'Projects.list': 'GET /api/projects',
  'Projects.create': 'POST /api/projects',
  'Projects.get': 'GET /api/projects/{project_id}',
  'Projects.update': 'PUT /api/projects/{project_id}',
  'Projects.delete': 'DELETE /api/projects/{project_id}',
  'Projects.uploadFile': 'POST /api/projects/{project_id}/files',
  'Projects.listFiles': 'GET /api/projects/{project_id}/files',

  // File Management
  'Files.upload': 'POST /api/files/upload',
  'Files.get': 'GET /api/files/{id}',
  'Files.delete': 'DELETE /api/files/{id}',
  'Files.download': 'GET /api/files/{id}/download',
  'Files.generateDownloadToken': 'POST /api/files/{id}/download-token',
  'Files.downloadWithToken': 'GET /api/files/{id}/download-with-token',
  'Files.preview': 'GET /api/files/{id}/preview',

  // Hub
  'Hub.getData': 'GET /api/hub/data',
  'Hub.refresh': 'POST /api/hub/refresh',
  'Hub.getVersion': 'GET /api/hub/version',
  'Hub.getModelReadme': 'GET /api/hub/models/{model_id}/readme',

  // User Provider Management
  'Providers.list': 'GET /api/providers',
  'Providers.listProviderModels': 'GET /api/providers/{provider_id}/models',

  // ===========================
  // ADMIN ENDPOINTS
  // ===========================

  // Admin - User Management
  'Admin.listUsers': 'GET /api/admin/users',
  'Admin.getUser': 'GET /api/admin/users/{user_id}',
  'Admin.updateUser': 'PUT /api/admin/users/{user_id}',
  'Admin.toggleUserActive': 'POST /api/admin/users/{user_id}/toggle-active',
  'Admin.resetPassword': 'POST /api/admin/users/reset-password',

  // Admin - User Group Management
  'Admin.listGroups': 'GET /api/admin/groups',
  'Admin.createGroup': 'POST /api/admin/groups',
  'Admin.getGroup': 'GET /api/admin/groups/{group_id}',
  'Admin.updateGroup': 'PUT /api/admin/groups/{group_id}',
  'Admin.deleteGroup': 'DELETE /api/admin/groups/{group_id}',
  'Admin.getGroupMembers': 'GET /api/admin/groups/{group_id}/members',
  'Admin.assignUserToGroup': 'POST /api/admin/groups/assign',
  'Admin.removeUserFromGroup':
    'DELETE /api/admin/groups/{user_id}/{group_id}/remove',

  // Admin - Group Provider Relationships
  'Admin.getGroupProviders': 'GET /api/admin/groups/{group_id}/providers',
  'Admin.assignProviderToGroup': 'POST /api/admin/groups/assign-provider',
  'Admin.removeProviderFromGroup':
    'DELETE /api/admin/groups/{group_id}/providers/{provider_id}',
  'Admin.getProviderGroups': 'GET /api/admin/providers/{provider_id}/groups',
  'Admin.listUserGroupProviderRelationships':
    'GET /api/admin/user-group-provider-relationships',

  // Admin - Assistant Management
  'Admin.listAssistants': 'GET /api/admin/assistants',
  'Admin.createAssistant': 'POST /api/admin/assistants',
  'Admin.getAssistant': 'GET /api/admin/assistants/{assistant_id}',
  'Admin.updateAssistant': 'PUT /api/admin/assistants/{assistant_id}',
  'Admin.deleteAssistant': 'DELETE /api/admin/assistants/{assistant_id}',

  // Admin - Provider Management
  'Admin.listProviders': 'GET /api/admin/providers',
  'Admin.getProvider': 'GET /api/admin/providers/{provider_id}',
  'Admin.createProvider': 'POST /api/admin/providers',
  'Admin.updateProvider': 'PUT /api/admin/providers/{provider_id}',
  'Admin.deleteProvider': 'DELETE /api/admin/providers/{provider_id}',
  'Admin.cloneProvider': 'POST /api/admin/providers/{provider_id}/clone',
  'Admin.addModelToProvider': 'POST /api/admin/providers/{provider_id}/models',
  'Admin.listProviderModels': 'GET /api/admin/providers/{provider_id}/models',

  // Admin - Model Management
  'Admin.getModel': 'GET /api/admin/models/{model_id}',
  'Admin.updateModel': 'PUT /api/admin/models/{model_id}',
  'Admin.deleteModel': 'DELETE /api/admin/models/{model_id}',
  'Admin.startModel': 'POST /api/admin/models/{model_id}/start',
  'Admin.stopModel': 'POST /api/admin/models/{model_id}/stop',
  'Admin.enableModel': 'POST /api/admin/models/{model_id}/enable',
  'Admin.disableModel': 'POST /api/admin/models/{model_id}/disable',
  'Admin.getAvailableDevices': 'GET /api/admin/devices',

  // Admin - Model Upload & Repository Management
  'Admin.uploadAndCommitModel':
    'POST /api/admin/uploaded-models/upload-and-commit',
  'Admin.listRepositories': 'GET /api/admin/repositories',
  'Admin.getRepository': 'GET /api/admin/repositories/{repository_id}',
  'Admin.createRepository': 'POST /api/admin/repositories',
  'Admin.updateRepository': 'PUT /api/admin/repositories/{repository_id}',
  'Admin.deleteRepository': 'DELETE /api/admin/repositories/{repository_id}',
  'Admin.testRepositoryConnection': 'POST /api/admin/repositories/test',
  'Admin.downloadFromRepository':
    'POST /api/admin/models/download-from-repository',
  'Admin.initiateRepositoryDownload':
    'POST /api/admin/models/initiate-repository-download',

  // Admin - Download Management
  'Admin.listAllDownloads': 'GET /api/admin/downloads',
  'Admin.getDownload': 'GET /api/admin/downloads/{download_id}',
  'Admin.cancelDownload': 'POST /api/admin/downloads/{download_id}/cancel',
  'Admin.deleteDownload': 'DELETE /api/admin/downloads/{download_id}',
  'Admin.subscribeDownloadProgress': 'GET /api/admin/downloads/subscribe',

  // Admin - Configuration Management
  'Admin.getUserRegistrationStatus': 'GET /api/admin/config/user-registration',
  'Admin.updateUserRegistrationStatus':
    'PUT /api/admin/config/user-registration',
  'Admin.getDefaultLanguage': 'GET /api/admin/config/default-language',
  'Admin.updateDefaultLanguage': 'PUT /api/admin/config/default-language',
  'Admin.getProxySettings': 'GET /api/admin/config/proxy',
  'Admin.updateProxySettings': 'PUT /api/admin/config/proxy',
  'Admin.getNgrokSettings': 'GET /api/admin/config/ngrok',
  'Admin.updateNgrokSettings': 'PUT /api/admin/config/ngrok', 
  'Admin.startNgrokTunnel': 'POST /api/admin/config/ngrok/start',
  'Admin.stopNgrokTunnel': 'POST /api/admin/config/ngrok/stop',
  'Admin.getNgrokStatus': 'GET /api/admin/config/ngrok/status',
  'User.updateAccountPassword': 'PUT /api/admin/config/user/password',

  // Admin - Hardware Management
  'Admin.getHardwareInfo': 'GET /api/admin/hardware',
  'Admin.subscribeHardwareUsage': 'GET /api/admin/hardware/usage-stream',

  // Admin - Engine Management
  'Admin.listEngines': 'GET /api/admin/engines',

  // ===========================
  // RAG PROVIDER MANAGEMENT
  // ===========================

  // Admin - RAG Provider Management
  'Admin.listRAGProviders': 'GET /api/admin/rag-providers',
  'Admin.getRAGProvider': 'GET /api/admin/rag-providers/{provider_id}',
  'Admin.createRAGProvider': 'POST /api/admin/rag-providers',
  'Admin.updateRAGProvider': 'PUT /api/admin/rag-providers/{provider_id}',
  'Admin.deleteRAGProvider': 'DELETE /api/admin/rag-providers/{provider_id}',
  'Admin.cloneRAGProvider': 'POST /api/admin/rag-providers/{provider_id}/clone',

  // Admin - RAG Database Management
  'Admin.listRAGProviderDatabases':
    'GET /api/admin/rag-providers/{provider_id}/databases',
  'Admin.addDatabaseToRAGProvider':
    'POST /api/admin/rag-providers/{provider_id}/databases',
  'Admin.getRAGDatabase': 'GET /api/admin/rag-databases/{database_id}',
  'Admin.updateRAGDatabase': 'PUT /api/admin/rag-databases/{database_id}',
  'Admin.deleteRAGDatabase': 'DELETE /api/admin/rag-databases/{database_id}',
  'Admin.startRAGDatabase': 'POST /api/admin/rag-databases/{database_id}/start',
  'Admin.stopRAGDatabase': 'POST /api/admin/rag-databases/{database_id}/stop',
  'Admin.enableRAGDatabase':
    'POST /api/admin/rag-databases/{database_id}/enable',
  'Admin.disableRAGDatabase':
    'POST /api/admin/rag-databases/{database_id}/disable',

  // Admin - RAG Repository Management
  'Admin.listRAGRepositories': 'GET /api/admin/rag-repositories',
  'Admin.getRAGRepository': 'GET /api/admin/rag-repositories/{repository_id}',
  'Admin.createRAGRepository': 'POST /api/admin/rag-repositories',
  'Admin.updateRAGRepository':
    'PUT /api/admin/rag-repositories/{repository_id}',
  'Admin.deleteRAGRepository':
    'DELETE /api/admin/rag-repositories/{repository_id}',
  'Admin.testRAGRepositoryConnection':
    'POST /api/admin/rag-repositories/{repository_id}/test-connection',
  'Admin.listRAGRepositoryDatabases':
    'GET /api/admin/rag-repositories/{repository_id}/databases',
  'Admin.downloadRAGDatabaseFromRepository':
    'POST /api/admin/rag-repositories/download-database',
} as const

// Define parameters for each endpoint - TypeScript will ensure all endpoints are covered
export type ApiEndpointParameters = {
  'App.getHttpPort': void
  'Auth.init': void
  'Auth.setup': CreateUserRequest
  'Auth.login': LoginRequest
  'Auth.logout': void
  'Auth.register': CreateUserRequest
  'Auth.me': void
  'User.greet': void
  // Admin user management
  'Admin.listUsers': { page?: number; per_page?: number }
  'Admin.getUser': { user_id: string }
  'Admin.updateUser': UpdateUserRequest
  'Admin.toggleUserActive': { user_id: string }
  'Admin.resetPassword': ResetPasswordRequest
  // Admin group management
  'Admin.listGroups': { page?: number; per_page?: number }
  'Admin.createGroup': CreateUserGroupRequest
  'Admin.getGroup': { group_id: string }
  'Admin.updateGroup': UpdateUserGroupRequest
  'Admin.deleteGroup': { group_id: string }
  'Admin.getGroupMembers': {
    group_id: string
    page?: number
    per_page?: number
  }
  'Admin.assignUserToGroup': AssignUserToGroupRequest
  'Admin.removeUserFromGroup': { user_id: string; group_id: string }
  // User Group Provider relationships
  'Admin.getGroupProviders': { group_id: string }
  'Admin.assignProviderToGroup': { group_id: string; provider_id: string }
  'Admin.removeProviderFromGroup': {
    group_id: string
    provider_id: string
  }
  'Admin.getProviderGroups': { provider_id: string }
  'Admin.listUserGroupProviderRelationships': void
  // Public configuration
  'Config.getUserRegistrationStatus': void
  'Config.getDefaultLanguage': void
  // Admin configuration management
  'Admin.getUserRegistrationStatus': void
  'Admin.updateUserRegistrationStatus': UpdateUserRegistrationRequest
  'Admin.getDefaultLanguage': void
  'Admin.updateDefaultLanguage': UpdateDefaultLanguageRequest
  'Admin.getProxySettings': void
  'Admin.updateProxySettings': UpdateProxySettingsRequest
  'Admin.getNgrokSettings': void
  'Admin.updateNgrokSettings': UpdateNgrokSettingsRequest
  'Admin.startNgrokTunnel': void
  'Admin.stopNgrokTunnel': void
  'Admin.getNgrokStatus': void
  'User.updateAccountPassword': UpdateAccountPasswordRequest
  // Admin hardware management
  'Admin.getHardwareInfo': void
  'Admin.subscribeHardwareUsage': void
  // Admin engine management
  'Admin.listEngines': void

  'Utils.testProxy': TestProxyConnectionRequest
  // User settings management
  'UserSettings.getAll': void
  'UserSettings.get': { key: string }
  'UserSettings.set': UserSettingRequest
  'UserSettings.delete': { key: string }
  'UserSettings.deleteAll': void
  // Admin provider management
  'Admin.listProviders': { page?: number; per_page?: number }
  'Admin.getProvider': { provider_id: string }
  'Admin.createProvider': CreateProviderRequest
  'Admin.updateProvider': { provider_id: string } & UpdateProviderRequest
  'Admin.deleteProvider': { provider_id: string }
  'Admin.cloneProvider': { provider_id: string }
  'Admin.addModelToProvider': {
    provider_id: string
  } & AddModelToProviderRequest
  'Admin.listProviderModels': { provider_id: string }
  'Admin.getModel': { model_id: string }
  'Admin.updateModel': { model_id: string } & UpdateModelRequest
  'Admin.deleteModel': { model_id: string }
  'Admin.startModel': { model_id: string }
  'Admin.stopModel': { model_id: string }
  'Admin.enableModel': { model_id: string }
  'Admin.disableModel': { model_id: string }
  'Admin.getAvailableDevices': void
  // Admin Model Upload parameters
  'Admin.uploadAndCommitModel': FormData
  // Assistant endpoints - User
  'Assistant.list': { page?: number; per_page?: number }
  'Assistant.create': CreateAssistantRequest
  'Assistant.get': { assistant_id: string }
  'Assistant.update': { assistant_id: string } & UpdateAssistantRequest
  'Assistant.delete': { assistant_id: string }
  'Assistant.getDefault': void
  // Assistant endpoints - Admin
  'Admin.listAssistants': { page?: number; per_page?: number }
  'Admin.createAssistant': CreateAssistantRequest
  'Admin.getAssistant': { assistant_id: string }
  'Admin.updateAssistant': { assistant_id: string } & UpdateAssistantRequest
  'Admin.deleteAssistant': { assistant_id: string }
  // Chat endpoints
  'Chat.listConversations': {
    page?: number
    per_page?: number
    project_id?: string
  }
  'Chat.createConversation': CreateConversationRequest
  'Chat.getConversation': { conversation_id: string }
  'Chat.updateConversation': {
    conversation_id: string
  } & UpdateConversationRequest
  'Chat.deleteConversation': { conversation_id: string }
  'Chat.sendMessageStream': SendMessageRequest
  'Chat.editMessageStream': { message_id: string } & SendMessageRequest
  'Chat.getMessageBranches': { message_id: string }
  'Chat.getConversationMessages': {
    conversation_id: string
    branch_id: string
  }
  'Chat.switchConversationBranch': {
    conversation_id: string
  } & SwitchBranchRequest
  'Chat.searchConversations': {
    q: string
    page?: number
    per_page?: number
    project_id?: string
  }
  // Project endpoints
  'Projects.list': ProjectListParams
  'Projects.create': CreateProjectRequest
  'Projects.get': { project_id: string }
  'Projects.update': { project_id: string } & UpdateProjectRequest
  'Projects.delete': { project_id: string }
  'Projects.uploadFile': FormData
  'Projects.listFiles': { project_id: string } & FileListParams
  // File endpoints
  'Files.upload': FormData
  'Files.get': { id: string }
  'Files.delete': { id: string }
  'Files.download': { id: string }
  'Files.generateDownloadToken': { id: string }
  'Files.downloadWithToken': { id: string; token: string }
  'Files.preview': { id: string; page?: number }
  // Repository endpoints - Admin only (all repository operations are admin-only)
  // Admin repository endpoints
  'Admin.listRepositories': { page?: number; per_page?: number }
  'Admin.getRepository': { repository_id: string }
  'Admin.createRepository': CreateRepositoryRequest
  'Admin.updateRepository': { repository_id: string } & UpdateRepositoryRequest
  'Admin.deleteRepository': { repository_id: string }
  'Admin.testRepositoryConnection': TestRepositoryConnectionRequest
  'Admin.downloadFromRepository': {
    provider_id: string
    repository_id: string
    repository_path: string
    main_filename: string
    repository_branch?: string
    name: string
    alias: string
    description?: string
    file_format: string
    capabilities?: ModelCapabilities
    parameters?: ModelParameters
    engine_type?: string
    engine_settings_mistralrs?: MistralRsSettings
    engine_settings_llamacpp?: LlamaCppSettings
  }
  'Admin.initiateRepositoryDownload': DownloadFromRepositoryRequest
  // Download instance endpoints - Admin (all download operations are admin-only)
  'Admin.listAllDownloads': {
    page?: number
    per_page?: number
    status?: string
  }
  'Admin.getDownload': { download_id: string }
  'Admin.cancelDownload': { download_id: string }
  'Admin.deleteDownload': { download_id: string }
  'Admin.subscribeDownloadProgress': void
  // Hub endpoints
  'Hub.getData': { lang?: string }
  'Hub.refresh': { lang?: string }
  'Hub.getVersion': void
  'Hub.getModelReadme': { model_id: string }
  // User Provider endpoints
  'Providers.list': { page?: number; per_page?: number }
  'Providers.listProviderModels': { provider_id: string }

  // RAG Provider Management - Parameters
  'Admin.listRAGProviders': { page?: number; per_page?: number }
  'Admin.getRAGProvider': { provider_id: string }
  'Admin.createRAGProvider': CreateRAGProviderRequest
  'Admin.updateRAGProvider': { provider_id: string } & UpdateRAGProviderRequest
  'Admin.deleteRAGProvider': { provider_id: string }
  'Admin.cloneRAGProvider': { provider_id: string }

  // RAG Database Management - Parameters
  'Admin.listRAGProviderDatabases': { provider_id: string }
  'Admin.addDatabaseToRAGProvider': {
    provider_id: string
  } & CreateRAGDatabaseRequest
  'Admin.getRAGDatabase': { database_id: string }
  'Admin.updateRAGDatabase': { database_id: string } & UpdateRAGDatabaseRequest
  'Admin.deleteRAGDatabase': { database_id: string }
  'Admin.startRAGDatabase': { database_id: string }
  'Admin.stopRAGDatabase': { database_id: string }
  'Admin.enableRAGDatabase': { database_id: string }
  'Admin.disableRAGDatabase': { database_id: string }

  // RAG Repository Management - Parameters
  'Admin.listRAGRepositories': { page?: number; per_page?: number }
  'Admin.getRAGRepository': { repository_id: string }
  'Admin.createRAGRepository': CreateRAGRepositoryRequest
  'Admin.updateRAGRepository': {
    repository_id: string
  } & UpdateRAGRepositoryRequest
  'Admin.deleteRAGRepository': { repository_id: string }
  'Admin.testRAGRepositoryConnection': { repository_id: string }
  'Admin.listRAGRepositoryDatabases': { repository_id: string }
  'Admin.downloadRAGDatabaseFromRepository': DownloadRAGDatabaseFromRepositoryRequest
}

// Define responses for each endpoint - TypeScript will ensure all endpoints are covered
export type ApiEndpointResponses = {
  'App.getHttpPort': number
  'Auth.init': InitResponse
  'Auth.setup': AuthResponse
  'Auth.login': AuthResponse
  'Auth.logout': void
  'Auth.register': AuthResponse
  'Auth.me': User
  'User.greet': void
  // Admin user management
  'Admin.listUsers': UserListResponse
  'Admin.getUser': User
  'Admin.updateUser': User
  'Admin.toggleUserActive': { is_active: boolean }
  'Admin.resetPassword': void
  // Admin group management
  'Admin.listGroups': UserGroupListResponse
  'Admin.createGroup': UserGroup
  'Admin.getGroup': UserGroup
  'Admin.updateGroup': UserGroup
  'Admin.deleteGroup': void
  'Admin.getGroupMembers': UserListResponse
  'Admin.assignUserToGroup': void
  'Admin.removeUserFromGroup': void
  // User Group Provider relationships
  'Admin.getGroupProviders': Provider[]
  'Admin.assignProviderToGroup': {
    id: string
    group_id: string
    provider_id: string
    assigned_at: string
    provider: Provider
    group: UserGroup
  }
  'Admin.removeProviderFromGroup': void
  'Admin.getProviderGroups': UserGroup[]
  'Admin.listUserGroupProviderRelationships': {
    id: string
    group_id: string
    provider_id: string
    assigned_at: string
    provider: Provider
    group: UserGroup
  }[]
  // Public configuration
  'Config.getUserRegistrationStatus': UserRegistrationStatusResponse
  'Config.getDefaultLanguage': DefaultLanguageResponse
  // Admin configuration management
  'Admin.getUserRegistrationStatus': UserRegistrationStatusResponse
  'Admin.updateUserRegistrationStatus': UserRegistrationStatusResponse
  'Admin.getDefaultLanguage': DefaultLanguageResponse
  'Admin.updateDefaultLanguage': DefaultLanguageResponse
  'Admin.getProxySettings': ProxySettingsResponse
  'Admin.updateProxySettings': ProxySettingsResponse
  'Admin.getNgrokSettings': NgrokSettingsResponse
  'Admin.updateNgrokSettings': NgrokSettingsResponse
  'Admin.startNgrokTunnel': NgrokStatusResponse
  'Admin.stopNgrokTunnel': NgrokStatusResponse 
  'Admin.getNgrokStatus': NgrokStatusResponse
  'User.updateAccountPassword': void
  // Admin hardware management
  'Admin.getHardwareInfo': HardwareInfoResponse
  'Admin.subscribeHardwareUsage': void // SSE endpoint
  // Admin engine management
  'Admin.listEngines': EngineListResponse
  // Document extraction configuration
  'Utils.testProxy': TestProxyConnectionResponse
  // User settings management
  'UserSettings.getAll': UserSettingsResponse
  'UserSettings.get': UserSetting
  'UserSettings.set': UserSetting
  'UserSettings.delete': void
  'UserSettings.deleteAll': { deleted: number }
  // Admin provider management
  'Admin.listProviders': ProviderListResponse
  'Admin.getProvider': Provider
  'Admin.createProvider': Provider
  'Admin.updateProvider': Provider
  'Admin.deleteProvider': void
  'Admin.cloneProvider': Provider
  'Admin.addModelToProvider': Model
  'Admin.listProviderModels': Model[]
  'Admin.getModel': Model
  'Admin.updateModel': Model
  'Admin.deleteModel': void
  'Admin.startModel': void
  'Admin.stopModel': void
  'Admin.enableModel': void
  'Admin.disableModel': void
  'Admin.getAvailableDevices': AvailableDevicesResponse
  // Admin Model Upload responses
  'Admin.uploadAndCommitModel': Model
  // Assistant endpoints - User
  'Assistant.list': AssistantListResponse
  'Assistant.create': Assistant
  'Assistant.get': Assistant
  'Assistant.update': Assistant
  'Assistant.delete': void
  'Assistant.getDefault': Assistant
  // Assistant endpoints - Admin
  'Admin.listAssistants': AssistantListResponse
  'Admin.createAssistant': Assistant
  'Admin.getAssistant': Assistant
  'Admin.updateAssistant': Assistant
  'Admin.deleteAssistant': void
  // Chat endpoints
  'Chat.listConversations': ConversationListResponse
  'Chat.createConversation': Conversation
  'Chat.getConversation': Conversation
  'Chat.updateConversation': Conversation
  'Chat.deleteConversation': void
  'Chat.sendMessageStream': any // Streaming response
  'Chat.editMessageStream': any // Streaming response
  'Chat.getMessageBranches': MessageBranch[]
  'Chat.getConversationMessages': Message[]
  'Chat.switchConversationBranch': { success: boolean; message: string }
  'Chat.searchConversations': ConversationListResponse
  // Project endpoints
  'Projects.list': ProjectListResponse
  'Projects.create': Project
  'Projects.get': ProjectDetailResponse
  'Projects.update': Project
  'Projects.delete': void
  'Projects.uploadFile': UploadFileResponse
  'Projects.listFiles': FileListResponse
  // File endpoints
  'Files.upload': UploadFileResponse
  'Files.get': File
  'Files.delete': void
  'Files.download': Blob
  'Files.generateDownloadToken': DownloadTokenResponse
  'Files.downloadWithToken': Blob
  'Files.preview': Blob
  // Repository endpoints - Admin only (all repository operations are admin-only)
  // Admin repository endpoints
  'Admin.listRepositories': RepositoryListResponse
  'Admin.getRepository': Repository
  'Admin.createRepository': Repository
  'Admin.updateRepository': Repository
  'Admin.deleteRepository': void
  'Admin.testRepositoryConnection': TestRepositoryConnectionResponse
  'Admin.downloadFromRepository': Model
  'Admin.initiateRepositoryDownload': DownloadInstance
  // Download instance endpoints - Admin (all download operations are admin-only)
  'Admin.listAllDownloads': DownloadInstanceListResponse
  'Admin.getDownload': DownloadInstance
  'Admin.cancelDownload': void
  'Admin.deleteDownload': void
  'Admin.subscribeDownloadProgress': any // SSE stream
  // Hub endpoints
  'Hub.getData': HubDataResponse
  'Hub.refresh': HubDataResponse
  'Hub.getVersion': HubVersionResponse
  'Hub.getModelReadme': { content: string }
  // User Provider endpoints
  'Providers.list': ProviderListResponse
  'Providers.listProviderModels': Model[]

  // RAG Provider Management - Responses
  'Admin.listRAGProviders': RAGProviderListResponse
  'Admin.getRAGProvider': RAGProvider
  'Admin.createRAGProvider': RAGProvider
  'Admin.updateRAGProvider': RAGProvider
  'Admin.deleteRAGProvider': void
  'Admin.cloneRAGProvider': RAGProvider

  // RAG Database Management - Responses
  'Admin.listRAGProviderDatabases': RAGDatabase[]
  'Admin.addDatabaseToRAGProvider': RAGDatabase
  'Admin.getRAGDatabase': RAGDatabase
  'Admin.updateRAGDatabase': RAGDatabase
  'Admin.deleteRAGDatabase': void
  'Admin.startRAGDatabase': void
  'Admin.stopRAGDatabase': void
  'Admin.enableRAGDatabase': void
  'Admin.disableRAGDatabase': void

  // RAG Repository Management - Responses
  'Admin.listRAGRepositories': RAGRepositoryListResponse
  'Admin.getRAGRepository': RAGRepository
  'Admin.createRAGRepository': RAGRepository
  'Admin.updateRAGRepository': RAGRepository
  'Admin.deleteRAGRepository': void
  'Admin.testRAGRepositoryConnection': RAGRepositoryConnectionTestResponse
  'Admin.listRAGRepositoryDatabases': RAGDatabase[]
  'Admin.downloadRAGDatabaseFromRepository': RAGDatabase
}

// Type helpers
export type ApiEndpoint = keyof typeof ApiEndpoints
export type ApiEndpointUrl = (typeof ApiEndpoints)[ApiEndpoint]

// Create reverse mapping from URL to endpoint key
export type UrlToEndpoint<U extends ApiEndpointUrl> = {
  [K in keyof typeof ApiEndpoints]: (typeof ApiEndpoints)[K] extends U
    ? K
    : never
}[keyof typeof ApiEndpoints]

// Helper types to get parameter and response types by URL
export type ParameterByUrl<U extends ApiEndpointUrl> =
  ApiEndpointParameters[UrlToEndpoint<U>]
export type ResponseByUrl<U extends ApiEndpointUrl> =
  ApiEndpointResponses[UrlToEndpoint<U>]

// Type-safe validation - this will cause a TypeScript error if any endpoint is missing
type ValidateParametersComplete = {
  [K in keyof typeof ApiEndpoints]: K extends keyof ApiEndpointParameters
    ? true
    : false
}

type ValidateResponsesComplete = {
  [K in keyof typeof ApiEndpoints]: K extends keyof ApiEndpointResponses
    ? true
    : false
}

// Type-safe validation - these will cause a TypeScript error if any endpoint is missing
// from Parameters or Responses. They are used for compile-time validation only.
export type { ValidateParametersComplete, ValidateResponsesComplete }
