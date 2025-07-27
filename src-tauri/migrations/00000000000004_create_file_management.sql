-- ===============================
-- FILE MANAGEMENT SYSTEM
-- ===============================

-- Create files table
CREATE TABLE files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    filename VARCHAR(255) NOT NULL,
    file_path VARCHAR(500) NOT NULL,
    file_size BIGINT NOT NULL,
    mime_type VARCHAR(100),
    checksum VARCHAR(64),
    project_id UUID REFERENCES projects(id) ON DELETE CASCADE,
    thumbnail_count INTEGER DEFAULT 0,
    processing_metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create messages_files table for many-to-many relationship
CREATE TABLE messages_files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    file_id UUID NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(message_id, file_id)
);

-- Create provider_files table for provider-specific file mappings
CREATE TABLE provider_files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    file_id UUID NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    provider_id UUID NOT NULL REFERENCES providers(id) ON DELETE CASCADE,
    provider_file_id VARCHAR(255),
    provider_metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(file_id, provider_id)
);

-- ===============================
-- INDEXES FOR FILES
-- ===============================

-- Files indexes
CREATE INDEX idx_files_user_id ON files(user_id);
CREATE INDEX idx_files_project_id ON files(project_id);
CREATE INDEX idx_files_mime_type ON files(mime_type);
CREATE INDEX idx_files_created_at ON files(created_at DESC);
CREATE INDEX idx_files_file_size ON files(file_size);
CREATE INDEX idx_files_checksum ON files(checksum);
CREATE INDEX idx_files_processing_metadata ON files USING GIN(processing_metadata);

-- Messages-files relationship indexes
CREATE INDEX idx_messages_files_message_id ON messages_files(message_id);
CREATE INDEX idx_messages_files_file_id ON messages_files(file_id);

-- Provider-files relationship indexes
CREATE INDEX idx_provider_files_file_id ON provider_files(file_id);
CREATE INDEX idx_provider_files_provider_id ON provider_files(provider_id);
CREATE INDEX idx_provider_files_provider_file_id ON provider_files(provider_file_id);
CREATE INDEX idx_provider_files_metadata ON provider_files USING GIN(provider_metadata);

-- ===============================
-- TRIGGERS AND FUNCTIONS
-- ===============================

-- Trigger for files updated_at
CREATE TRIGGER update_files_updated_at
    BEFORE UPDATE ON files
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ===============================
-- TABLE COMMENTS
-- ===============================

COMMENT ON TABLE files IS 'Files table for storing uploaded files for projects and chat messages';
COMMENT ON TABLE messages_files IS 'Many-to-many relationship between messages and files';
COMMENT ON TABLE provider_files IS 'Provider-specific file mappings for LLM compatibility';

-- Column comments
COMMENT ON COLUMN files.filename IS 'Original filename with extension';
COMMENT ON COLUMN files.file_path IS 'Path to original file in filesystem';
COMMENT ON COLUMN files.file_size IS 'File size in bytes';
COMMENT ON COLUMN files.mime_type IS 'MIME type of the file';
COMMENT ON COLUMN files.checksum IS 'SHA-256 hash of the file for integrity verification';
COMMENT ON COLUMN files.project_id IS 'Project this file belongs to (nullable for general uploads)';
COMMENT ON COLUMN files.thumbnail_count IS 'Number of thumbnails generated for this file';
COMMENT ON COLUMN files.processing_metadata IS 'Processing results and metadata (text length, pages, dimensions, etc.)';
COMMENT ON COLUMN provider_files.provider_file_id IS 'Provider-specific file ID if applicable';
COMMENT ON COLUMN provider_files.provider_metadata IS 'Provider-specific metadata and processing info';