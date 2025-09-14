# RAG Query API Endpoint Implementation Plan

## Overview

This plan outlines the implementation of a single backend API endpoint to query RAG instances using the simple_vector engine. This endpoint is for testing RAG query functionality and is not integrated with the chat system.

**Updated**: Plan now uses the existing `RAGSource` struct instead of `SimpleVectorDocumentResult`, which provides enhanced functionality including entity matches and relationship matches for future knowledge graph capabilities.

## Existing API Pattern Analysis

Based on analysis of `/src-tauri/src/api/rag/instances.rs` and `/src-tauri/src/route/rag.rs`:

### Standard Authentication Pattern
```rust
Extension(auth_user): Extension<AuthenticatedUser>,
Path(instance_id): Path<Uuid>,
```

### Standard Permission Validation
```rust
let has_access = validate_rag_instance_access(auth_user.user.id, instance_id, false)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::from(e)))?;
if !has_access {
    return Err((StatusCode::FORBIDDEN, AppError::forbidden("Access denied")));
}
```

### Standard Error Handling
```rust
.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::from(e)))?
```

## Implementation Plan

### 1. Request/Response Structures

Create new structures in `/src-tauri/src/api/rag/instances.rs`:

```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RAGQueryRequest {
    /// The query text to search for
    pub query: String,
    /// Maximum number of results to return (optional, default: 10)
    pub max_results: Option<usize>,
    /// Enable reranking of results (optional, default: false)  
    pub enable_rerank: Option<bool>,
    /// Similarity threshold for vector search (optional, default: 0.7)
    pub similarity_threshold: Option<f32>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct RAGQueryResponse {
    /// RAG search results with similarity scores and entity matches
    pub results: Vec<RAGSource>,
    /// Token usage statistics
    pub token_usage: RAGTokenUsage,
    /// Processing metadata
    pub metadata: RAGQueryMetadata,
}

// Note: RAGSource and SimpleVectorDocument are already defined in the RAG module:
// use crate::ai::rag::{RAGSource, engines::simple_vector::types::SimpleVectorDocument};
//
// RAGSource structure:
// pub struct RAGSource {
//     pub document: SimpleVectorDocument,
//     pub similarity_score: f32,
//     pub entity_matches: Vec<String>,
//     pub relationship_matches: Vec<String>,
// }

#[derive(Debug, Serialize, JsonSchema)]
pub struct RAGTokenUsage {
    /// Total tokens used in query processing
    pub total_tokens: u32,
    /// Tokens used for embedding queries
    pub embedding_tokens: u32,
    /// Maximum tokens that were budgeted
    pub max_total_tokens: u32,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct RAGQueryMetadata {
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Number of chunks retrieved before filtering
    pub chunks_retrieved: usize,
    /// Number of chunks after similarity filtering
    pub chunks_filtered: usize,
    /// Whether reranking was applied
    pub rerank_applied: bool,
}
```

### 2. Handler Implementation

Add to `/src-tauri/src/api/rag/instances.rs`:

```rust
/// Query RAG instance for testing purposes
#[debug_handler]
pub async fn query_rag_instance_handler(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(instance_id): Path<Uuid>,
    Json(request): Json<RAGQueryRequest>,
) -> ApiResult<Json<RAGQueryResponse>> {
    let start_time = std::time::Instant::now();
    
    // Validate user has access to this RAG instance
    let has_access = validate_rag_instance_access(auth_user.user.id, instance_id, false)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::from(e)))?;
    if !has_access {
        return Err((StatusCode::FORBIDDEN, AppError::forbidden("Access denied")));
    }

    // Get RAG instance details
    let instance = get_rag_instance_by_id(instance_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::from(e)))?
        .ok_or((StatusCode::NOT_FOUND, AppError::not_found("RAG instance")))?;

    // Always use Retrieval mode for this testing endpoint
    let mode = QueryMode::Retrieval;

    let max_results = request.max_results.unwrap_or(10).min(50); // Cap at 50
    let similarity_threshold = request.similarity_threshold.unwrap_or(0.7).clamp(0.0, 1.0);

    // Build query context
    let context = QueryContext {
        mode,
        max_results,
        similarity_threshold,
        enable_rerank: request.enable_rerank.unwrap_or(false),
        conversation_history: Vec::new(), // Empty for testing
        file_ids: None, // Query all files in instance
        stream: false, // No streaming for testing API
    };

    // Get RAG service and execute query
    let rag_service = RAGService::get_instance()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error(&format!("RAG service error: {}", e))))?;

    let rag_response = rag_service
        .query(instance_id, &request.query, context)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error(&format!("Query failed: {}", e))))?;

    let processing_time = start_time.elapsed().as_millis() as u64;

    // The results are already in RAGSource format from the RAG engine
    let results = rag_response.sources;

    let token_usage = RAGTokenUsage {
        total_tokens: rag_response.metadata.get("total_tokens")
            .and_then(|v| v.as_u64()).unwrap_or(0) as u32,
        embedding_tokens: rag_response.metadata.get("embedding_tokens")
            .and_then(|v| v.as_u64()).unwrap_or(0) as u32,
        max_total_tokens: rag_response.metadata.get("max_total_tokens")
            .and_then(|v| v.as_u64()).unwrap_or(0) as u32,
    };

    let metadata = RAGQueryMetadata {
        processing_time_ms: processing_time,
        chunks_retrieved: rag_response.metadata.get("chunks_retrieved")
            .and_then(|v| v.as_u64()).unwrap_or(0) as usize,
        chunks_filtered: rag_response.metadata.get("chunks_filtered")
            .and_then(|v| v.as_u64()).unwrap_or(0) as usize,
        rerank_applied: request.enable_rerank.unwrap_or(false),
    };

    let response = RAGQueryResponse {
        results,
        token_usage,
        metadata,
    };

    Ok((StatusCode::OK, Json(response)))
}
```

### 3. Route Registration

Uncomment and update in `/src-tauri/src/route/rag.rs`:

```rust
// Add to existing user_rag_routes() function
.api_route(
    "/instances/{instance_id}/query",
    post_with(instances::query_rag_instance_handler, |op| {
        op.description("Query RAG instance for testing (not for chat integration)")
            .id("Rag.queryInstance")
            .tag("rag")
            .response::<200, Json<RAGQueryResponse>>()
            .response_with::<400, Json<AppError>>(|res| res.description("Invalid request parameters"))
            .response_with::<403, Json<AppError>>(|res| res.description("Access denied to RAG instance"))
            .response_with::<404, Json<AppError>>(|res| res.description("RAG instance not found"))
            .response_with::<500, Json<AppError>>(|res| res.description("Internal server error"))
    })
    .layer(middleware::from_fn(
        crate::api::middleware::permissions::rag_instances_read_middleware,
    )),
)
```

### 4. Required Imports

Add to `/src-tauri/src/api/rag/instances.rs`:

```rust
use crate::ai::rag::{
    service::{RAGService, QueryContext},
    QueryMode, RAGSource,
    engines::simple_vector::types::SimpleVectorDocument,
};
use crate::database::queries::rag_instances::get_rag_instance_by_id;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use sqlx::FromRow;
```

### 5. Error Handling Strategy

Follow existing patterns:
- Use `StatusCode::BAD_REQUEST` for validation errors
- Use `StatusCode::FORBIDDEN` for access denied
- Use `StatusCode::NOT_FOUND` for missing resources  
- Use `StatusCode::INTERNAL_SERVER_ERROR` for system errors
- Always include meaningful error messages in `AppError`

### 6. API Endpoint Details

**Endpoint**: `POST /api/rag/instances/{instance_id}/query`

**Authentication**: Required (JWT token)

**Permissions**: `rag_instances_read_middleware` 

**Request Body**:
```json
{
    "query": "What is the capital of France?",
    "max_results": 10,
    "enable_rerank": false,
    "similarity_threshold": 0.7
}
```

**Response Example**:
```json
{
    "results": [
        {
            "document": {
                "id": "123e4567-e89b-12d3-a456-426614174000",
                "rag_instance_id": "550e8400-e29b-41d4-a716-446655440000",
                "file_id": "550e8400-e29b-41d4-a716-446655440001",
                "chunk_index": 5,
                "content": "Paris is the capital and most populous city of France...",
                "content_hash": "abc123def456",
                "token_count": 15,
                "metadata": {},
                "created_at": "2024-09-10T10:00:00Z",
                "updated_at": "2024-09-10T10:00:00Z"
            },
            "similarity_score": 0.95,
            "entity_matches": ["Paris", "France", "capital"],
            "relationship_matches": ["capital_of", "located_in"]
        }
    ],
    "token_usage": {
        "total_tokens": 20,
        "embedding_tokens": 20,
        "max_total_tokens": 4000
    },
    "metadata": {
        "processing_time_ms": 245,
        "chunks_retrieved": 25,
        "chunks_filtered": 5,
        "rerank_applied": false
    }
}
```

## Implementation Notes

### Integration Points

1. **RAG Service**: Uses existing `RAGService::get_instance()` pattern
2. **Database**: Leverages existing `validate_rag_instance_access` for permissions
3. **Authentication**: Uses standard `AuthenticatedUser` middleware
4. **Error Handling**: Follows existing `AppError` patterns
5. **RAG Types**: Uses existing `RAGSource` struct with entity and relationship matching capabilities

### Testing Approach

1. **Unit Testing**: Test request validation and response formatting
2. **Integration Testing**: Test with actual RAG instances and queries  
3. **Permission Testing**: Verify access control works correctly
4. **Performance Testing**: Monitor response times and token usage

### Future Considerations

This endpoint is designed for testing retrieval functionality and can be extended later for:
- Generation mode integration (when connecting with chat system)  
- Streaming responses (when integrating with chat)
- Advanced query parameters (filters, metadata queries)
- Batch query processing
- Query result caching
- Entity and relationship extraction integration (leveraging the existing RAGSource structure)
- Knowledge graph querying capabilities

## Implementation Checklist

- [ ] Add request/response structures to `instances.rs`
- [ ] Implement `query_rag_instance_handler` function
- [ ] Add required imports for RAG service integration
- [ ] Uncomment and configure route in `rag.rs`
- [ ] Test compilation with `cargo build`
- [ ] Test endpoint with bearer token and actual RAG instance
- [ ] Validate error handling for all edge cases
- [ ] Verify permission middleware works correctly

This plan provides a single, focused endpoint for testing RAG query functionality while following all established patterns in the existing codebase.