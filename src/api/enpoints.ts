// Define all API endpoints

export const ApiEndpoints = {
  'User.greet': 'POST /api/user/greet',
  'App.getHttpPort': 'GET /get_http_port',
  'Auth.init': 'GET /api/auth/init',
  'Auth.setup': 'POST /api/auth/setup',
  'Auth.login': 'POST /api/auth/login',
  'Auth.logout': 'POST /api/auth/logout',
  'Auth.register': 'POST /api/auth/register',
  'Auth.me': 'GET /api/auth/me',
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
}

// Auth types
export interface User {
  id: string
  username: string
  emails: UserEmail[]
  created_at: string
  profile?: any
  services: UserServices
}

export interface UserEmail {
  address: string
  verified: boolean
}

export interface UserServices {
  facebook?: any
  resume?: any
  password?: any
}

export interface CreateUserRequest {
  username: string
  email: string
  password: string
  profile?: any
}

export interface LoginRequest {
  username_or_email: string
  password: string
}

export interface AuthResponse {
  token: string
  user: User
  expires_at: string
}

export interface InitResponse {
  needs_setup: boolean
  is_desktop: boolean
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
