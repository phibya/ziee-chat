# RAGSource Refactoring Plan

## Overview

Refactor `RAGSource` to use `SimpleVectorDocument` instead of duplicating fields like `file_id`, `filename`, `chunk_index`, and `content_snippet`. This will improve consistency, reduce duplication, and provide complete document information.

## Current State Analysis

### Current RAGSource Structure
```rust
// In src-tauri/src/ai/rag/mod.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGSource {
    pub file_id: Uuid,
    pub filename: String,
    pub chunk_index: Option<usize>,
    pub content_snippet: String,
    pub similarity_score: f32,
    pub entity_matches: Vec<String>,
    pub relationship_matches: Vec<String>,
}
```

### Usage Locations
1. **RAGQueryResponse.sources** (`src/ai/rag/mod.rs:377`) - Used as return type for query responses
2. **SimpleVectorEngine.build_retrieval_response()** (`src/ai/rag/engines/simple_vector/core.rs`) - Creates RAGSource from VectorSearchResult
3. **SimpleVectorEngine.build_generation_response()** (`src/ai/rag/engines/simple_vector/core.rs`) - Creates RAGSource from VectorSearchResult

### Current Problems
1. **Duplication**: `RAGSource` duplicates fields that exist in `SimpleVectorDocument` 
2. **Inconsistency**: `filename` is hardcoded as `format!("file_{}", chunk.file_id)` with TODO comment
3. **Limited Information**: Only provides truncated `content_snippet` instead of full document data
4. **Type Mismatch**: `chunk_index` is `Option<usize>` in RAGSource but `i32` in database

## Proposed Solution

### 1. Create SimpleVectorDocument in simple_vector/types.rs

Add `SimpleVectorDocument` struct to `src-tauri/src/ai/rag/engines/simple_vector/types.rs`:

```rust
// In src-tauri/src/ai/rag/engines/simple_vector/types.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sqlx::FromRow;

/// Simple vector document matching simple_vector_documents table
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SimpleVectorDocument {
    pub id: Uuid,
    pub rag_instance_id: Uuid,
    pub file_id: Uuid,
    pub chunk_index: i32,
    pub content: String,
    pub content_hash: String,
    pub token_count: i32,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### 2. Refactor RAGSource Structure

Update `RAGSource` in `src-tauri/src/ai/rag/mod.rs`:

```rust
// In src-tauri/src/ai/rag/mod.rs
use crate::ai::rag::engines::simple_vector::types::SimpleVectorDocument;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGSource {
    pub document: SimpleVectorDocument,
    pub similarity_score: f32,
    pub entity_matches: Vec<String>,
    pub relationship_matches: Vec<String>,
}
```

### 3. Create Database Query for SimpleVectorDocument

Replace the current `VectorSearchResult` approach with a proper query that constructs `SimpleVectorDocument` from the database:

```rust
// In src-tauri/src/ai/rag/engines/simple_vector/queries.rs

/// Perform similarity search returning complete SimpleVectorDocument with similarity scores
pub async fn similarity_search_documents(
    instance_id: Uuid,
    query_embedding: &[f32],
    top_k: usize,
    similarity_threshold: Option<f32>,
    file_ids: Option<Vec<Uuid>>,
) -> RAGResult<Vec<(SimpleVectorDocument, f32)>> {
    let database = get_database_pool().map_err(|e| {
        tracing::error!("Failed to get database pool for similarity search: {}", e);
        RAGErrorCode::Querying(RAGQueryingErrorCode::SimilaritySearchFailed)
    })?;

    let query_vector = HalfVector::from_f32_slice(query_embedding);
    
    let results = if let Some(file_ids) = file_ids {
        // Search with file ID filtering
        sqlx::query!(
            r#"
            SELECT 
                id,
                rag_instance_id,
                file_id, 
                chunk_index, 
                content, 
                content_hash,
                token_count,
                metadata,
                created_at,
                updated_at,
                (1 - (embedding <=> $1::halfvec))::float4 as "similarity_score!"
            FROM simple_vector_documents 
            WHERE rag_instance_id = $2
              AND file_id = ANY($3)
              AND ($4::float4 IS NULL OR 1 - (embedding <=> $1::halfvec) >= $4)
            ORDER BY embedding <=> $1::halfvec
            LIMIT $5
            "#,
            query_vector as HalfVector,
            instance_id,
            &file_ids[..],
            similarity_threshold,
            top_k as i64
        )
        .fetch_all(&*database)
        .await
    } else {
        // Search all documents in instance
        sqlx::query!(
            r#"
            SELECT 
                id,
                rag_instance_id,
                file_id, 
                chunk_index, 
                content, 
                content_hash,
                token_count,
                metadata,
                created_at,
                updated_at,
                (1 - (embedding <=> $1::halfvec))::float4 as "similarity_score!"
            FROM simple_vector_documents 
            WHERE rag_instance_id = $2
              AND ($3::float4 IS NULL OR 1 - (embedding <=> $1::halfvec) >= $3)
            ORDER BY embedding <=> $1::halfvec
            LIMIT $4
            "#,
            query_vector as HalfVector,
            instance_id,
            similarity_threshold,
            top_k as i64
        )
        .fetch_all(&*database)
        .await
    };

    let rows = results.map_err(|e| {
        tracing::error!(
            "Failed to execute similarity search for instance {}: {}",
            instance_id,
            e
        );
        RAGErrorCode::Querying(RAGQueryingErrorCode::SimilaritySearchFailed)
    })?;

    // Convert database rows to (SimpleVectorDocument, similarity_score) tuples
    let documents_with_scores: Vec<(SimpleVectorDocument, f32)> = rows
        .into_iter()
        .map(|row| {
            let document = SimpleVectorDocument {
                id: row.id,
                rag_instance_id: row.rag_instance_id,
                file_id: row.file_id,
                chunk_index: row.chunk_index,
                content: row.content,
                content_hash: row.content_hash,
                token_count: row.token_count,
                metadata: row.metadata,
                created_at: row.created_at,
                updated_at: row.updated_at,
            };
            (document, row.similarity_score)
        })
        .collect();

    Ok(documents_with_scores)
}
```

### 4. Update SimpleVectorEngine Methods

Modify both response building methods in `src-tauri/src/ai/rag/engines/simple_vector/core.rs`:

#### build_retrieval_response()
```rust
fn build_retrieval_response(&self, documents_with_scores: Vec<(SimpleVectorDocument, f32)>, processing_time: u64) -> RAGQueryResponse {
    let sources: Vec<RAGSource> = documents_with_scores.into_iter().map(|(document, similarity_score)| {
        RAGSource {
            document,
            similarity_score,
            entity_matches: Vec::new(), // TODO: Implement entity extraction
            relationship_matches: Vec::new(), // TODO: Implement relationship extraction
        }
    }).collect();
    
    let mut metadata = HashMap::new();
    metadata.insert("chunks_retrieved".to_string(), serde_json::json!(sources.len()));
    
    RAGQueryResponse {
        answer: String::new(), // Empty for retrieval-only
        sources,
        mode_used: QueryMode::Retrieval,
        confidence_score: None,
        processing_time_ms: processing_time,
        metadata,
    }
}
```

#### build_generation_response()
```rust
fn build_generation_response(&self, answer: String, documents_with_scores: Vec<(SimpleVectorDocument, f32)>, processing_time: u64, mode: QueryMode, token_budget: &TokenBudget) -> RAGQueryResponse {
    let sources: Vec<RAGSource> = documents_with_scores.into_iter().map(|(document, similarity_score)| {
        RAGSource {
            document,
            similarity_score,
            entity_matches: Vec::new(), // TODO: Implement entity extraction
            relationship_matches: Vec::new(), // TODO: Implement relationship extraction
        }
    }).collect();
    
    // Include token budget information in metadata for debugging/monitoring
    let mut metadata = HashMap::new();
    metadata.insert("chunks_used".to_string(), serde_json::json!(sources.len()));
    metadata.insert("max_total_tokens".to_string(), serde_json::json!(token_budget.max_total_tokens));
    // ... other token metadata
    
    RAGQueryResponse {
        answer,
        sources,
        mode_used: mode,
        confidence_score: None, // TODO: Calculate confidence
        processing_time_ms: processing_time,
        metadata,
    }
}
```

### 5. Update Query Methods

Update the main query method in `SimpleVectorEngine`:

```rust
// In core.rs query() method
let documents_with_scores = similarity_search_documents(
    instance_id,
    &embedding,
    context.max_results,
    Some(context.similarity_threshold),
    context.file_ids.clone()
).await?;

match context.mode {
    QueryMode::Retrieval => {
        Ok(self.build_retrieval_response(documents_with_scores, processing_time))
    },
    QueryMode::Generation | QueryMode::Naive | QueryMode::Local => {
        // Use the documents for context formatting
        let llm_response = self.generate_llm_response(/* ... */).await?;
        Ok(self.build_generation_response(llm_response, documents_with_scores, processing_time, context.mode, &token_budget))
    },
    // ...
}
```

## Implementation Steps

### Phase 1: Core Structure Updates
1. **Add SimpleVectorDocument** to `src-tauri/src/ai/rag/engines/simple_vector/types.rs`
2. **Update RAGSource** in `src-tauri/src/ai/rag/mod.rs` to use SimpleVectorDocument
3. **Add re-export** in `src-tauri/src/ai/rag/engines/simple_vector/mod.rs`

### Phase 2: Database Query Updates
1. **Create new similarity_search_documents function** in `queries.rs` that returns `(SimpleVectorDocument, f32)` tuples
2. **Update existing similarity_search** to use new function internally
3. **Add filename resolution** - either join with files table or fetch separately

### Phase 3: Engine Method Updates
1. **Update build_retrieval_response()** to accept and use `Vec<(SimpleVectorDocument, f32)>`
2. **Update build_generation_response()** to accept and use `Vec<(SimpleVectorDocument, f32)>`
3. **Update main query() method** to use new data types

### Phase 4: Testing and Validation
1. **Compile and test** the simple_vector engine
2. **Verify RAGQueryResponse** serialization works correctly
3. **Test API endpoint** (when implemented) with new structure

## Benefits

1. **Consistency**: RAGSource now uses the actual database structure
2. **Complete Information**: Full document data available, not just snippets
3. **Type Safety**: Proper types matching database schema
4. **Maintainability**: Single source of truth for document structure
5. **Extensibility**: Easy to add new document fields in future

## Breaking Changes

### JSON Response Format
The RAGQueryResponse JSON will change from:
```json
{
  "sources": [
    {
      "file_id": "uuid",
      "filename": "file_uuid", 
      "chunk_index": 5,
      "content_snippet": "truncated content...",
      "similarity_score": 0.95,
      "entity_matches": [],
      "relationship_matches": []
    }
  ]
}
```

To:
```json
{
  "sources": [
    {
      "document": {
        "id": "uuid",
        "rag_instance_id": "uuid",
        "file_id": "uuid",
        "chunk_index": 5,
        "content": "full content text...",
        "content_hash": "hash",
        "token_count": 150,
        "metadata": {},
        "created_at": "2024-09-10T10:00:00Z",
        "updated_at": "2024-09-10T10:00:00Z"
      },
      "similarity_score": 0.95,
      "entity_matches": [],
      "relationship_matches": []
    }
  ]
}
```

### Impact Assessment
- **Backend Impact**: Changes to RAG engine implementation but public query API remains the same
- **Frontend Impact**: Any frontend code consuming RAGQueryResponse will need updates
- **Database Impact**: No schema changes required - only query modifications

## Implementation Checklist

- [ ] Add SimpleVectorDocument to types.rs
- [ ] Update RAGSource structure
- [ ] Create similarity_search_documents function
- [ ] Update build_retrieval_response method
- [ ] Update build_generation_response method  
- [ ] Update main query method
- [ ] Add proper filename resolution
- [ ] Test compilation
- [ ] Test functionality
- [ ] Update API endpoint plan (if needed)

This refactoring improves the architecture by using proper domain types and providing complete document information while maintaining the existing RAG engine interface.