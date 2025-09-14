# RAG Query Files Enhancement Plan

## Overview
Add unique file information to RAGQueryResponse to provide file context for each search result. This enhancement will include the unique File records for all file_id values referenced in the RAGSource results.

## Current Structure Analysis

### Existing File Functionality (src/store/files.ts)
The codebase already has comprehensive file handling capabilities:

```typescript
// Core file operations available:
- getFile(fileId: string): Promise<File>
- getFileThumbnail(fileId: string): Promise<string | null>
- getFileThumbnails(fileId: string, thumbnailCount: number): Promise<string[]>
- getFileContent(fileId: string): Promise<string>
- generateFileDownloadToken(fileId: string): Promise<{ token: string; expires_at: string }>
- uploadFile(file: globalThis.File, progressCallback?: Function): Promise<File>
- deleteFile(fileId: string, projectId?: string): Promise<void>

// API endpoints available:
- Files.downloadFile: GET /api/files/{file_id}/download
- Files.downloadFileWithToken: GET /api/files/{file_id}/download-with-token
- Files.generateDownloadToken: POST /api/files/{file_id}/download-token
- Files.getFile: GET /api/files/{file_id}
- Files.getFilePreview: GET /api/files/{file_id}/preview
```

**Key capabilities to leverage:**
- Direct file download via `ApiClient.Files.downloadFile()`
- Secure download tokens via `generateFileDownloadToken()`
- File metadata access via `getFile()`
- No thumbnail requirement (as per user request)

### File Download Pattern (from src/components/common/FileCard.tsx)
The existing FileCard component shows the established download pattern:

```typescript
// 1. Generate download token on component load
useEffect(() => {
  generateFileDownloadToken(file.id)
    .then(({ token }) => {
      setDownloadToken(token)
    })
    .catch(error => {
      console.error('Failed to generate download token:', error)
      message.error('Failed to generate download link')
    })
}, [file.id])

// 2. Use token in download link (FileModalContent)
<a
  href={`/api/files/${file.id}/download-with-token?token=${downloadToken}`}
  download={file.filename}
  className="ant-btn ant-btn-primary"
>
  <DownloadOutlined /> Download File
</a>
```

**Key patterns to follow:**
- Generate tokens lazily (only when needed for download)
- Use `/api/files/{file_id}/download-with-token?token={token}` URL pattern
- Handle token generation errors gracefully
- Show user feedback via message.success/error

## Current RAG Structure Analysis

### RAGQueryResponse (src-tauri/src/api/rag/instances.rs)
```rust
#[derive(Debug, Serialize, JsonSchema)]
pub struct RAGQueryResponse {
    /// RAG search results with similarity scores and entity matches
    pub results: Vec<RAGSource>,
    /// Token usage statistics
    pub token_usage: RAGTokenUsage,
    /// Processing metadata
    pub metadata: RAGQueryMetadata,
}
```

### RAGSource (src-tauri/src/ai/rag/mod.rs)
```rust
pub struct RAGSource {
    pub document: SimpleVectorDocument,  // Contains file_id
    pub similarity_score: f32,
    pub entity_matches: Vec<String>,
    pub relationship_matches: Vec<String>,
}
```

### SimpleVectorDocument (src-tauri/src/ai/rag/models.rs)
```rust
pub struct SimpleVectorDocument {
    pub id: Uuid,
    pub rag_instance_id: Uuid,
    pub file_id: Uuid,  // File reference
    pub chunk_index: i32,
    pub content: String,
    // ... other fields
}
```

### File Model (src-tauri/src/database/models/file.rs)
```rust
pub struct File {
    pub id: Uuid,
    pub user_id: Uuid,
    pub filename: String,
    pub file_size: i64,
    pub mime_type: Option<String>,
    pub checksum: Option<String>,
    pub project_id: Option<Uuid>,
    pub thumbnail_count: i32,
    pub page_count: i32,
    pub processing_metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

## Proposed Changes

### 1. Update RAGQueryResponse Structure
**File:** `src-tauri/src/api/rag/instances.rs`

Add `files` field to RAGQueryResponse:
```rust
#[derive(Debug, Serialize, JsonSchema)]
pub struct RAGQueryResponse {
    /// RAG search results with similarity scores and entity matches
    pub results: Vec<RAGSource>,
    /// Unique files referenced in the search results
    pub files: Vec<File>,
    /// Token usage statistics
    pub token_usage: RAGTokenUsage,
    /// Processing metadata
    pub metadata: RAGQueryMetadata,
}
```

### 2. Import Required Dependencies
**File:** `src-tauri/src/api/rag/instances.rs`

Add import for File model:
```rust
use crate::database::models::file::File;
```

### 3. Create File Lookup Query Function
**File:** `src-tauri/src/database/queries/files.rs` (if not exists) or existing file queries

Add function to get files by IDs:
```rust
/// Get multiple files by their IDs
pub async fn get_files_by_ids(file_ids: Vec<Uuid>) -> Result<Vec<File>, sqlx::Error> {
    let database = get_database_pool()?;
    
    let files = sqlx::query_as!(
        File,
        r#"
        SELECT 
            id, user_id, filename, file_size, mime_type, checksum,
            project_id, thumbnail_count, page_count, processing_metadata,
            created_at, updated_at
        FROM files 
        WHERE id = ANY($1)
        ORDER BY filename
        "#,
        &file_ids[..]
    )
    .fetch_all(&*database)
    .await?;
    
    Ok(files)
}
```

### 4. Update Query Handler Logic
**File:** `src-tauri/src/api/rag/instances.rs`

Modify `query_rag_instance_handler` function:
```rust
pub async fn query_rag_instance_handler(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(instance_id): Path<Uuid>,
    Json(request): Json<RAGQueryRequest>,
) -> ApiResult<Json<RAGQueryResponse>> {
    // ... existing validation and RAG query logic ...
    
    // After getting rag_response, extract unique file IDs
    let unique_file_ids: Vec<Uuid> = rag_response.sources
        .iter()
        .map(|source| source.document.file_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    
    // Fetch file information
    let files = if !unique_file_ids.is_empty() {
        crate::database::queries::files::get_files_by_ids(unique_file_ids)
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch files for RAG query: {}", e);
                ApiError::InternalServer("Failed to fetch file information".to_string())
            })?
    } else {
        Vec::new()
    };
    
    // Build final response
    let response = RAGQueryResponse {
        results: rag_response.sources,
        files,
        token_usage: RAGTokenUsage {
            total_tokens: rag_response.metadata
                .get("total_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
            embedding_tokens: 0, // TODO: Extract from metadata
            max_total_tokens: rag_response.metadata
                .get("max_total_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
        },
        metadata: RAGQueryMetadata {
            processing_time_ms: rag_response.processing_time_ms,
            chunks_retrieved: rag_response.sources.len(),
            chunks_after_filtering: rag_response.sources.len(),
            rerank_applied: rag_response.metadata
                .get("rerank_applied")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        },
    };
    
    Ok(Json(response))
}
```

### 5. Update Frontend Types
**File:** `src/types/api/rag.ts` (if exists) or relevant frontend type definitions

Update TypeScript interface:
```typescript
export interface RAGQueryResponse {
  results: RAGSource[]
  files: File[]  // Add this field
  token_usage: RAGTokenUsage
  metadata: RAGQueryMetadata
}

export interface File {
  id: string
  user_id: string
  filename: string
  file_size: number
  mime_type: string | null
  checksum: string | null
  project_id: string | null
  thumbnail_count: number
  page_count: number
  processing_metadata: Record<string, any>
  created_at: string
  updated_at: string
}
```

### 6. Update Frontend Components
**File:** `src/components/Pages/Rags/RagQueryTab.tsx`

Enhance the results display to show file information with download capability:

```typescript
import { generateFileDownloadToken } from '../../../store/files'
import { DownloadOutlined } from '@ant-design/icons'

// Add file lookup helper
const getFileInfo = (fileId: string) => {
  return queryResults?.files.find(file => file.id === fileId)
}

// Add download handler (following FileCard.tsx pattern)
const handleFileDownload = async (fileId: string, filename: string) => {
  try {
    const { token } = await generateFileDownloadToken(fileId)
    
    // Use the same pattern as FileCard.tsx - direct navigation with token
    const downloadUrl = `/api/files/${fileId}/download-with-token?token=${token}`
    window.open(downloadUrl, '_blank')
    
    message.success(`Downloading ${filename}`)
  } catch (error) {
    message.error('Failed to download file')
    console.error('Download error:', error)
  }
}

// Update SourceCard component to display filename with download option
const SourceCard: React.FC<{ source: RAGSource; index: number }> = ({
  source,
  index,
}) => {
  const { document, similarity_score, entity_matches, relationship_matches } = source
  const fileInfo = getFileInfo(document.file_id)
  
  return (
    <Card
      size="small"
      title={
        <Flex justify="space-between" align="center">
          <Text strong>#{index + 1}</Text>
          <Text type="secondary" className="text-sm font-normal">
            Similarity: {formatSimilarity(similarity_score)}
          </Text>
        </Flex>
      }
      className="mb-3"
    >
      <TruncatedContent content={document.content} />
      
      <div className="mt-3 pt-3">
        <Space direction="vertical" size="small" className="w-full">
          <Flex justify="space-between" align="center">
            <Text type="secondary" className="text-xs">
              File: {fileInfo?.filename || 'Unknown'} | 
              Chunk: {document.chunk_index} | 
              Tokens: {document.token_count}
            </Text>
            {fileInfo && (
              <Button
                type="link"
                size="small"
                icon={<DownloadOutlined />}
                onClick={() => handleFileDownload(fileInfo.id, fileInfo.filename)}
                className="p-0 h-auto"
                title={`Download ${fileInfo.filename}`}
              >
                Download
              </Button>
            )}
          </Flex>
          
          {/* File metadata display */}
          {fileInfo && (
            <Text type="secondary" className="text-xs">
              Size: {(fileInfo.file_size / 1024).toFixed(1)} KB | 
              Type: {fileInfo.mime_type || 'Unknown'} | 
              Pages: {fileInfo.page_count}
            </Text>
          )}
          
          // ... entity and relationship matches
        </Space>
      </div>
    </Card>
  )
}
```

## Implementation Insights from FileCard Analysis

After analyzing `src/components/common/FileCard.tsx`, key insights for RAG query file enhancement:

### Download Implementation Patterns
1. **Lazy Token Generation**: FileCard generates download tokens only when the modal opens, not for every file in list view
2. **Direct URL Navigation**: Uses `window.open()` or direct `<a>` tags with `href` for downloads
3. **Error Handling**: Comprehensive error handling with user feedback via messages
4. **File Type Handling**: Different behavior for text files vs other files

### UI/UX Patterns
1. **File Extension Display**: Shows file extension in small badge (e.g., "PDF", "DOCX")
2. **File Size Display**: Uses `formatFileSize()` utility for human-readable sizes
3. **Download Icon**: Consistent use of `<DownloadOutlined />` icon
4. **Button Styling**: Link-type buttons for minimal visual impact

### Performance Considerations
1. **Token Caching**: Tokens are cached in component state to avoid repeated API calls
2. **Lazy Loading**: Thumbnails and tokens only loaded when needed
3. **Cleanup**: Proper cleanup of object URLs to prevent memory leaks

### Recommended RAG Implementation Strategy
1. **Minimal Token Generation**: Only generate tokens when user clicks download (not on result load)
2. **Consistent UI**: Match FileCard visual patterns for familiarity
3. **File Context**: Show file extension and size alongside download option
4. **Error Resilience**: Handle missing files gracefully with fallback display

## Implementation Steps

### Phase 1: Backend Structure Updates
1. Update RAGQueryResponse struct in `src-tauri/src/api/rag/instances.rs`
2. Add File model import
3. Create or update file query function for batch lookup
4. Update JsonSchema derives as needed

### Phase 2: Query Handler Logic
1. Modify `query_rag_instance_handler` to extract unique file IDs
2. Add database query to fetch file information
3. Include files in response construction
4. Test with existing RAG query endpoints

### Phase 3: Frontend Integration
1. Update TypeScript types
2. Modify RAG query components to use file information
3. Update UI to display filenames instead of file IDs
4. Add file-based filtering/grouping capabilities (future enhancement)

### Phase 4: Testing & Optimization
1. Test with various RAG queries containing different numbers of files
2. Performance testing for queries with many unique files
3. Ensure proper error handling when files are not found
4. Add integration tests for the enhanced response structure

## Benefits

1. **Enhanced User Experience**: Display meaningful filenames instead of UUIDs
2. **Better Context**: Users can understand which files contributed to results
3. **File Metadata Access**: Access to file size, type, and other metadata
4. **Future Extensibility**: Foundation for file-based result filtering/grouping

## Considerations

### Performance
- Additional database query for file lookup
- Consider caching frequently accessed files
- Optimize for queries with many unique files

### Error Handling
- Handle cases where files are deleted but chunks remain
- Graceful degradation when file info is unavailable

### Security
- Ensure proper authorization for file access
- Validate user permissions for returned files

### Data Consistency
- Consider adding file information validation
- Handle edge cases with orphaned chunks

## Testing Strategy

1. **Unit Tests**: File ID extraction and deduplication logic
2. **Integration Tests**: Complete query flow with file lookup
3. **Performance Tests**: Queries with 1, 10, 50+ unique files
4. **Edge Cases**: Missing files, permission issues, empty results

## Enhanced UI Design

Based on existing file functionality, the enhanced RAG query results will provide:

### Source Card Enhancements
1. **File Information Display**: Show filename instead of UUID
2. **Download Button**: Direct download capability using existing token system
3. **File Metadata**: Display file size, type, and page count
4. **No Thumbnails**: Focus on textual information and download functionality

### File Management Integration
- Leverage existing `src/store/files.ts` functions
- Use secure download token system for file access
- Maintain consistency with existing file UI patterns
- Provide contextual file information without UI bloat

### User Experience Improvements
- **Context**: Users understand which files contributed to results
- **Access**: Direct download capability for source files
- **Information**: File metadata for better understanding
- **Consistency**: Matches existing file handling patterns

## Future Enhancements

1. **File Grouping**: Group results by source file
2. **File Filtering**: Filter results by file type or name
3. **File Ranking**: Rank files by relevance across chunks
4. **File Content Preview**: Show file content using existing `getFileContent()`
5. **Bulk Download**: Download all referenced files in a query