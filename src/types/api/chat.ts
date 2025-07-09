/**
 * Chat API types - matching backend structure
 */

export interface Conversation {
  id: string
  title: string
  user_id: string
  assistant_id?: string
  model_provider_id?: string
  model_id?: string
  created_at: string
  updated_at: string
  messages: Message[]
}

export interface Message {
  id: string
  conversation_id: string
  parent_message_id?: string
  content: string
  role: 'user' | 'assistant' | 'system'
  branch_index: number
  created_at: string
  branches: Message[]
}

export interface CreateConversationRequest {
  title: string
  assistant_id?: string
  model_provider_id?: string
  model_id?: string
}

export interface UpdateConversationRequest {
  title?: string
  assistant_id?: string
  model_provider_id?: string
  model_id?: string
}

export interface SendMessageRequest {
  conversation_id: string
  content: string
  parent_message_id?: string
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
  model_provider_id?: string
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
