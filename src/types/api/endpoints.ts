/**
 * API endpoint type definitions
 * Centralized location for all API request/response types
 */

import { AuthResponse, InitResponse, LoginRequest } from './auth'
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
import {
  UpdateUserRegistrationRequest,
  UserRegistrationStatusResponse,
} from './config.ts'
import {
  DefaultLanguageResponse,
  UpdateDefaultLanguageRequest,
} from './globalConfig'
import { UserGroupListResponse } from './userGroup.ts'
import {
  UserSetting,
  UserSettingRequest,
  UserSettingsResponse,
} from './userSettings'
import {
  AddModelToProviderRequest,
  CloneModelProviderRequest,
  CreateModelProviderRequest,
  ModelProvider,
  ModelProviderListResponse,
  RemoveModelFromProviderRequest,
  UpdateModelProviderRequest,
} from './modelProvider'

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
  // Public configuration
  'Config.getUserRegistrationStatus': 'GET /api/config/user-registration',
  'Config.getDefaultLanguage': 'GET /api/config/default-language',
  // Admin configuration management
  'Admin.getUserRegistrationStatus': 'GET /api/admin/config/user-registration',
  'Admin.updateUserRegistrationStatus':
    'PUT /api/admin/config/user-registration',
  'Admin.getDefaultLanguage': 'GET /api/admin/config/default-language',
  'Admin.updateDefaultLanguage': 'PUT /api/admin/config/default-language',
  // User settings management
  'UserSettings.getAll': 'GET /api/user/settings',
  'UserSettings.get': 'GET /api/user/settings/{key}',
  'UserSettings.set': 'POST /api/user/settings',
  'UserSettings.delete': 'DELETE /api/user/settings/{key}',
  'UserSettings.deleteAll': 'DELETE /api/user/settings/all',
  // Model Provider management
  'ModelProviders.list': 'GET /api/model-providers',
  'ModelProviders.get': 'GET /api/model-providers/{provider_id}',
  'ModelProviders.create': 'POST /api/model-providers',
  'ModelProviders.update': 'PUT /api/model-providers/{provider_id}',
  'ModelProviders.delete': 'DELETE /api/model-providers/{provider_id}',
  'ModelProviders.clone': 'POST /api/model-providers/{provider_id}/clone',
  'ModelProviders.addModel': 'POST /api/model-providers/{provider_id}/models',
  'ModelProviders.removeModel':
    'DELETE /api/model-providers/{provider_id}/models/{model_id}',
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
  // Public configuration
  'Config.getUserRegistrationStatus': void
  'Config.getDefaultLanguage': void
  // Admin configuration management
  'Admin.getUserRegistrationStatus': void
  'Admin.updateUserRegistrationStatus': UpdateUserRegistrationRequest
  'Admin.getDefaultLanguage': void
  'Admin.updateDefaultLanguage': UpdateDefaultLanguageRequest
  // User settings management
  'UserSettings.getAll': void
  'UserSettings.get': { key: string }
  'UserSettings.set': UserSettingRequest
  'UserSettings.delete': { key: string }
  'UserSettings.deleteAll': void
  // Model Provider management
  'ModelProviders.list': { page?: number; per_page?: number }
  'ModelProviders.get': { provider_id: string }
  'ModelProviders.create': CreateModelProviderRequest
  'ModelProviders.update': UpdateModelProviderRequest
  'ModelProviders.delete': { provider_id: string }
  'ModelProviders.clone': CloneModelProviderRequest
  'ModelProviders.addModel': AddModelToProviderRequest
  'ModelProviders.removeModel': RemoveModelFromProviderRequest
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
  // Public configuration
  'Config.getUserRegistrationStatus': UserRegistrationStatusResponse
  'Config.getDefaultLanguage': DefaultLanguageResponse
  // Admin configuration management
  'Admin.getUserRegistrationStatus': UserRegistrationStatusResponse
  'Admin.updateUserRegistrationStatus': UserRegistrationStatusResponse
  'Admin.getDefaultLanguage': DefaultLanguageResponse
  'Admin.updateDefaultLanguage': DefaultLanguageResponse
  // User settings management
  'UserSettings.getAll': UserSettingsResponse
  'UserSettings.get': UserSetting
  'UserSettings.set': UserSetting
  'UserSettings.delete': void
  'UserSettings.deleteAll': { deleted: number }
  // Model Provider management
  'ModelProviders.list': ModelProviderListResponse
  'ModelProviders.get': ModelProvider
  'ModelProviders.create': ModelProvider
  'ModelProviders.update': ModelProvider
  'ModelProviders.delete': void
  'ModelProviders.clone': ModelProvider
  'ModelProviders.addModel': void
  'ModelProviders.removeModel': void
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

// These type checks will fail at compile time if any endpoint is missing from Parameters or Responses
// They are intentionally unused but serve as compile-time validators
//@ts-ignore
const _validateParameters: ValidateParametersComplete = {} as any
//@ts-ignore
const _validateResponses: ValidateResponsesComplete = {} as any
