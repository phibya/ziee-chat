/**
 * API endpoint type definitions
 * Centralized location for all API request/response types
 */

import {
  Assistant,
  AssistantListResponse,
  CreateAssistantRequest,
  UpdateAssistantRequest,
} from "./assistant";
import { AuthResponse, InitResponse, LoginRequest } from "./auth";
import {
  Conversation,
  ConversationListResponse,
  CreateConversationRequest,
  EditMessageRequest,
  Message,
  MessageBranch,
  SendMessageRequest,
  SwitchBranchRequest,
  UpdateConversationRequest,
} from "./chat";
import {
  ProxySettingsResponse,
  TestProxyConnectionRequest,
  TestProxyConnectionResponse,
  UpdateProxySettingsRequest,
  UpdateUserRegistrationRequest,
  UserRegistrationStatusResponse,
} from "./config.ts";
import {
  DefaultLanguageResponse,
  UpdateDefaultLanguageRequest,
} from "./globalConfig";
import {
  AddModelToProviderRequest,
  Model,
  ModelCapabilities,
  ModelSettings,
  UpdateModelRequest,
} from "./model";
import {
  CreateProjectRequest,
  Project,
  ProjectConversation,
  ProjectDetailResponse,
  ProjectListParams,
  ProjectListResponse,
  UpdateProjectRequest,
  UploadDocumentRequest,
  UploadDocumentResponse,
} from "./projects";
import {
  AvailableDevicesResponse,
  CreateProviderRequest,
  Provider,
  ProviderListResponse,
  UpdateProviderRequest,
} from "./provider";
import {
  CreateRepositoryRequest,
  Repository,
  RepositoryListResponse,
  TestRepositoryConnectionRequest,
  TestRepositoryConnectionResponse,
  UpdateRepositoryRequest,
} from "./repository";
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
} from "./user";
import { UserGroupListResponse } from "./userGroup.ts";
import {
  UserSetting,
  UserSettingRequest,
  UserSettingsResponse,
} from "./userSettings";

// API endpoint definitions
export const ApiEndpoints = {
  'User.greet': 'POST /api/user/greet',
  'App.getHttpPort': 'GET /get_http_port',
  'Auth.init': 'GET /api/auth/init',
  'Auth.setup': 'POST /api/auth/setup',
  'Auth.login': 'POST /api/auth/login',
  'Auth.logout': 'POST /api/auth/logout',
  'Auth.register': 'POST /api/auth/register',
  'Auth.me': 'GET /api/auth/me',
  // Admin user management
  'Admin.listUsers': 'GET /api/admin/users',
  'Admin.getUser': 'GET /api/admin/users/{user_id}',
  'Admin.updateUser': 'PUT /api/admin/users/{user_id}',
  'Admin.toggleUserActive': 'POST /api/admin/users/{user_id}/toggle-active',
  'Admin.resetPassword': 'POST /api/admin/users/reset-password',
  // Admin group management
  'Admin.listGroups': 'GET /api/admin/groups',
  'Admin.createGroup': 'POST /api/admin/groups',
  'Admin.getGroup': 'GET /api/admin/groups/{group_id}',
  'Admin.updateGroup': 'PUT /api/admin/groups/{group_id}',
  'Admin.deleteGroup': 'DELETE /api/admin/groups/{group_id}',
  'Admin.getGroupMembers': 'GET /api/admin/groups/{group_id}/members',
  'Admin.assignUserToGroup': 'POST /api/admin/groups/assign',
  'Admin.removeUserFromGroup':
    'DELETE /api/admin/groups/{user_id}/{group_id}/remove',
  // User Group Provider relationships
  'Admin.getGroupProviders': 'GET /api/admin/groups/{group_id}/providers',
  'Admin.assignProviderToGroup': 'POST /api/admin/groups/assign-provider',
  'Admin.removeProviderFromGroup':
    'DELETE /api/admin/groups/{group_id}/providers/{provider_id}',
  'Admin.getProviderGroups': 'GET /api/admin/providers/{provider_id}/groups',
  'Admin.listUserGroupProviderRelationships':
    'GET /api/admin/user-group-provider-relationships',
  // Public configuration
  'Config.getUserRegistrationStatus': 'GET /api/config/user-registration',
  'Config.getDefaultLanguage': 'GET /api/config/default-language',
  // Admin configuration management
  'Admin.getUserRegistrationStatus': 'GET /api/admin/config/user-registration',
  'Admin.updateUserRegistrationStatus':
    'PUT /api/admin/config/user-registration',
  'Admin.getDefaultLanguage': 'GET /api/admin/config/default-language',
  'Admin.updateDefaultLanguage': 'PUT /api/admin/config/default-language',
  'Admin.getProxySettings': 'GET /api/admin/config/proxy',
  'Admin.updateProxySettings': 'PUT /api/admin/config/proxy',
  'Utils.testProxy': 'POST /api/utils/test-proxy',
  // User settings management
  'UserSettings.getAll': 'GET /api/user/settings',
  'UserSettings.get': 'GET /api/user/settings/{key}',
  'UserSettings.set': 'POST /api/user/settings',
  'UserSettings.delete': 'DELETE /api/user/settings/{key}',
  'UserSettings.deleteAll': 'DELETE /api/user/settings/all',
  // Provider management
  'Providers.list': 'GET /api/admin/providers',
  'Providers.get': 'GET /api/admin/providers/{provider_id}',
  'Providers.create': 'POST /api/admin/providers',
  'Providers.update': 'PUT /api/admin/providers/{provider_id}',
  'Providers.delete': 'DELETE /api/admin/providers/{provider_id}',
  'Providers.clone': 'POST /api/admin/providers/{provider_id}/clone',
  'Providers.addModel': 'POST /api/admin/providers/{provider_id}/models',
  'Providers.listModels': 'GET /api/admin/providers/{provider_id}/models',
  'Models.get': 'GET /api/admin/models/{model_id}',
  'Models.update': 'PUT /api/admin/models/{model_id}',
  'Models.delete': 'DELETE /api/admin/models/{model_id}',
  'Models.start': 'POST /api/admin/models/{model_id}/start',
  'Models.stop': 'POST /api/admin/models/{model_id}/stop',
  'Models.enable': 'POST /api/admin/models/{model_id}/enable',
  'Models.disable': 'POST /api/admin/models/{model_id}/disable',
  'Admin.getAvailableDevices': 'GET /api/admin/devices',
  // Model Upload endpoints for Local
  'ModelUploads.uploadAndCommit':
    'POST /api/admin/uploaded-models/upload-and-commit',
  // Assistant endpoints - User
  'Assistant.list': 'GET /api/assistants',
  'Assistant.create': 'POST /api/assistants',
  'Assistant.get': 'GET /api/assistants/{assistant_id}',
  'Assistant.update': 'PUT /api/assistants/{assistant_id}',
  'Assistant.delete': 'DELETE /api/assistants/{assistant_id}',
  'Assistant.getDefault': 'GET /api/assistants/default',
  // Assistant endpoints - Admin
  'Admin.listAssistants': 'GET /api/admin/assistants',
  'Admin.createAssistant': 'POST /api/admin/assistants',
  'Admin.getAssistant': 'GET /api/admin/assistants/{assistant_id}',
  'Admin.updateAssistant': 'PUT /api/admin/assistants/{assistant_id}',
  'Admin.deleteAssistant': 'DELETE /api/admin/assistants/{assistant_id}',
  // Chat endpoints
  'Chat.listConversations': 'GET /api/chat/conversations',
  'Chat.createConversation': 'POST /api/chat/conversations',
  'Chat.getConversation': 'GET /api/chat/conversations/{conversation_id}',
  'Chat.updateConversation': 'PUT /api/chat/conversations/{conversation_id}',
  'Chat.deleteConversation': 'DELETE /api/chat/conversations/{conversation_id}',
  'Chat.sendMessage': 'POST /api/chat/messages/stream',
  'Chat.editMessage': 'PUT /api/chat/messages/{message_id}',
  'Chat.editMessageStream': 'PUT /api/chat/messages/{message_id}/stream',
  'Chat.getMessageBranches': 'GET /api/chat/messages/{message_id}/branches',
  'Chat.getConversationMessages':
    'GET /api/chat/conversations/{conversation_id}/messages/{branch_id}',
  'Chat.switchConversationBranch':
    'PUT /api/chat/conversations/{conversation_id}/branch/switch',
  'Chat.searchConversations': 'GET /api/chat/conversations/search',
  'Chat.clearAllConversations': 'DELETE /api/chat/conversations/clear-all',
  // Project endpoints
  'Projects.list': 'GET /api/projects',
  'Projects.create': 'POST /api/projects',
  'Projects.get': 'GET /api/projects/{project_id}',
  'Projects.update': 'PUT /api/projects/{project_id}',
  'Projects.delete': 'DELETE /api/projects/{project_id}',
  'Projects.uploadDocument': 'POST /api/projects/{project_id}/documents',
  'Projects.deleteDocument':
    'DELETE /api/projects/{project_id}/documents/{document_id}',
  'Projects.linkConversation':
    'POST /api/projects/{project_id}/conversations/{conversation_id}',
  'Projects.unlinkConversation':
    'DELETE /api/projects/{project_id}/conversations/{conversation_id}',
  // Repository endpoints
  'Repositories.list': 'GET /api/repositories',
  'Repositories.get': 'GET /api/repositories/{repository_id}',
  'Repositories.testConnection': 'POST /api/repositories/test',
  // Admin repository endpoints
  'Admin.listRepositories': 'GET /api/admin/repositories',
  'Admin.getRepository': 'GET /api/admin/repositories/{repository_id}',
  'Admin.createRepository': 'POST /api/admin/repositories',
  'Admin.updateRepository': 'PUT /api/admin/repositories/{repository_id}',
  'Admin.deleteRepository': 'DELETE /api/admin/repositories/{repository_id}',
  'Admin.testRepositoryConnection': 'POST /api/admin/repositories/test',
  'Admin.downloadFromRepository':
    'POST /api/admin/models/download-from-repository',
} as const

// Define parameters for each endpoint - TypeScript will ensure all endpoints are covered
export type ApiEndpointParameters = {
  'User.greet': { name: string }
  'App.getHttpPort': void
  'Auth.init': void
  'Auth.setup': CreateUserRequest
  'Auth.login': LoginRequest
  'Auth.logout': void
  'Auth.register': CreateUserRequest
  'Auth.me': void
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
  'Utils.testProxy': TestProxyConnectionRequest
  // User settings management
  'UserSettings.getAll': void
  'UserSettings.get': { key: string }
  'UserSettings.set': UserSettingRequest
  'UserSettings.delete': { key: string }
  'UserSettings.deleteAll': void
  // Provider management
  'Providers.list': { page?: number; per_page?: number }
  'Providers.get': { provider_id: string }
  'Providers.create': CreateProviderRequest
  'Providers.update': { provider_id: string } & UpdateProviderRequest
  'Providers.delete': { provider_id: string }
  'Providers.clone': { provider_id: string }
  'Providers.addModel': { provider_id: string } & AddModelToProviderRequest
  'Providers.listModels': { provider_id: string }
  'Models.get': { model_id: string }
  'Models.update': { model_id: string } & UpdateModelRequest
  'Models.delete': { model_id: string }
  'Models.start': { model_id: string }
  'Models.stop': { model_id: string }
  'Models.enable': { model_id: string }
  'Models.disable': { model_id: string }
  'Admin.getAvailableDevices': void
  // Model Upload parameters
  'ModelUploads.uploadAndCommit': FormData
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
  'Chat.listConversations': { page?: number; per_page?: number }
  'Chat.createConversation': CreateConversationRequest
  'Chat.getConversation': { conversation_id: string }
  'Chat.updateConversation': {
    conversation_id: string
  } & UpdateConversationRequest
  'Chat.deleteConversation': { conversation_id: string }
  'Chat.sendMessage': SendMessageRequest
  'Chat.editMessage': { message_id: string } & EditMessageRequest
  'Chat.editMessageStream': { message_id: string } & EditMessageRequest
  'Chat.getMessageBranches': { message_id: string }
  'Chat.getConversationMessages': {
    conversation_id: string
    branch_id: string
  }
  'Chat.switchConversationBranch': {
    conversation_id: string
  } & SwitchBranchRequest
  'Chat.searchConversations': { q: string; page?: number; per_page?: number }
  'Chat.clearAllConversations': void
  // Project endpoints
  'Projects.list': ProjectListParams
  'Projects.create': CreateProjectRequest
  'Projects.get': { project_id: string }
  'Projects.update': { project_id: string } & UpdateProjectRequest
  'Projects.delete': { project_id: string }
  'Projects.uploadDocument': { project_id: string } & UploadDocumentRequest
  'Projects.deleteDocument': { project_id: string; document_id: string }
  'Projects.linkConversation': { project_id: string; conversation_id: string }
  'Projects.unlinkConversation': { project_id: string; conversation_id: string }
  // Repository endpoints
  'Repositories.list': { page?: number; per_page?: number }
  'Repositories.get': { repository_id: string }
  'Repositories.testConnection': TestRepositoryConnectionRequest
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
    settings?: ModelSettings
  }
}

// Define responses for each endpoint - TypeScript will ensure all endpoints are covered
export type ApiEndpointResponses = {
  'User.greet': string
  'App.getHttpPort': number
  'Auth.init': InitResponse
  'Auth.setup': AuthResponse
  'Auth.login': AuthResponse
  'Auth.logout': void
  'Auth.register': AuthResponse
  'Auth.me': User
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
  'Utils.testProxy': TestProxyConnectionResponse
  // User settings management
  'UserSettings.getAll': UserSettingsResponse
  'UserSettings.get': UserSetting
  'UserSettings.set': UserSetting
  'UserSettings.delete': void
  'UserSettings.deleteAll': { deleted: number }
  // Provider management
  'Providers.list': ProviderListResponse
  'Providers.get': Provider
  'Providers.create': Provider
  'Providers.update': Provider
  'Providers.delete': void
  'Providers.clone': Provider
  'Providers.addModel': Model
  'Providers.listModels': Model[]
  'Models.get': Model
  'Models.update': Model
  'Models.delete': void
  'Models.start': { success: boolean; message: string }
  'Models.stop': { success: boolean; message: string }
  'Models.enable': { success: boolean; message: string }
  'Models.disable': { success: boolean; message: string }
  'Admin.getAvailableDevices': AvailableDevicesResponse
  // Model Upload responses
  'ModelUploads.uploadAndCommit': Model
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
  'Chat.sendMessage': any // Streaming response
  'Chat.editMessage': Message
  'Chat.editMessageStream': any // Streaming response
  'Chat.getMessageBranches': MessageBranch[]
  'Chat.getConversationMessages': Message[]
  'Chat.switchConversationBranch': { success: boolean; message: string }
  'Chat.searchConversations': ConversationListResponse
  'Chat.clearAllConversations': { deleted_count: number; message: string }
  // Project endpoints
  'Projects.list': ProjectListResponse
  'Projects.create': Project
  'Projects.get': ProjectDetailResponse
  'Projects.update': Project
  'Projects.delete': void
  'Projects.uploadDocument': UploadDocumentResponse
  'Projects.deleteDocument': void
  'Projects.linkConversation': ProjectConversation
  'Projects.unlinkConversation': void
  // Repository endpoints
  'Repositories.list': RepositoryListResponse
  'Repositories.get': Repository
  'Repositories.testConnection': TestRepositoryConnectionResponse
  // Admin repository endpoints
  'Admin.listRepositories': RepositoryListResponse
  'Admin.getRepository': Repository
  'Admin.createRepository': Repository
  'Admin.updateRepository': Repository
  'Admin.deleteRepository': void
  'Admin.testRepositoryConnection': TestRepositoryConnectionResponse
  'Admin.downloadFromRepository': Model
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
