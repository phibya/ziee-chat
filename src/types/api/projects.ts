export interface Project {
  id: string
  user_id: string
  name: string
  description?: string
  instruction?: string
  is_private: boolean
  document_count?: number
  conversation_count?: number
  created_at: string
  updated_at: string
}

export interface ProjectDocument {
  id: string
  project_id: string
  file_name: string
  file_path: string
  file_size: number
  mime_type?: string
  content_text?: string
  upload_status: string
  created_at: string
  updated_at: string
}

export interface ProjectConversation {
  id: string
  project_id: string
  conversation_id: string
  conversation?: any // Reference to Conversation type
  created_at: string
}

export interface CreateProjectRequest {
  name: string
  description?: string
  instruction?: string
  is_private?: boolean
}

export interface UpdateProjectRequest {
  name?: string
  description?: string
  instruction?: string
  is_private?: boolean
}

export interface ProjectListResponse {
  projects: Project[]
  total: number
  page: number
  per_page: number
}

export interface ProjectDetailResponse {
  project: Project
  documents: ProjectDocument[]
  conversations: ProjectConversation[]
}

export interface UploadDocumentRequest {
  file_name: string
  file_size: number
  mime_type?: string
}

export interface UploadDocumentResponse {
  document: ProjectDocument
  upload_url?: string
}

export interface ProjectListParams {
  page?: number
  per_page?: number
  search?: string
}
