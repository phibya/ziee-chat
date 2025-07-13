/**
 * Chat API types - matching backend structure
 */

export interface Conversation {
  id: string
  user_id: string
  title: string
  assistant_id: string
  model_id: string
  active_branch_id: string
  created_at: string
  updated_at: string
}

export interface Message {
  id: string
  conversation_id: string
  role: 'user' | 'assistant' | 'system'
  content: string
  originated_from_id?: string
  edit_count: number
  created_at: string
  updated_at: string
  metadata?: Array<{
    key: string
    value: any
  }>
}

export interface Branch {
  id: string
  conversation_id: string
  created_at: string
}

export interface MessageBranch {
  id: string
  conversation_id: string
  created_at: string
  is_clone: boolean
}

export interface CreateConversationRequest {
  title: string
  assistant_id?: string
  model_id?: string
}

export interface UpdateConversationRequest {
  title?: string
  assistant_id?: string
  model_id?: string
}

export interface SendMessageRequest {
  conversation_id: string
  content: string
  model_id: string
}

export interface SwitchBranchRequest {
  branch_id: string
}

export interface EditMessageRequest {
  content: string
}

export interface ConversationListResponse {
  conversations: ConversationSummary[]
  total: number
  page: number
  per_page: number
}

export interface ConversationSummary {
  id: string
  title: string
  user_id: string
  assistant_id?: string
  model_id?: string
  created_at: string
  updated_at: string
  last_message?: string
  message_count: number
}

export interface ChatResponse {
  message: Message
  conversation: Conversation
}
