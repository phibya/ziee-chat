import { User } from './user.ts'

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
  token?: string
}
