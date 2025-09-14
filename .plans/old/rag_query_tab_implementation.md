# RAG Query Interface Implementation Plan

Based on my investigation of the existing codebase, here's a comprehensive plan to implement the RAG query interface in `RagQueryTab.tsx`:

## **1. Architecture Overview**

### **Store Integration Pattern**
- **Store Access**: Use `Stores.RagInstance` for reactive RAG instance data
- **API Calls**: Add new query methods to RAG store following existing patterns
- **State Management**: Follow Zustand pattern with store methods defined outside store definition
- **Error Handling**: Use `App.useApp()` message system for notifications

### **Component Structure**
- **Form Pattern**: Use Ant Design `Form.useForm()` for query input
- **Results Display**: Card-based layout for query results with proper loading states
- **Responsive Design**: Follow existing layout patterns with proper breakpoints

## **2. Required Store Updates**

### **A. Add Query State to RAG Store (`src/store/rag.ts`)**
```typescript
interface RagState {
  // ... existing state
  
  // Query state
  queryResults: RAGQueryResponse | null
  querying: boolean
  lastQuery: string | null
  queryError: string | null
}
```

### **B. Add Query Methods**
```typescript
export const queryRAGInstance = async (
  instanceId: string, 
  queryData: RAGQueryRequest
): Promise<RAGQueryResponse> => {
  // Implementation following existing patterns
}

export const clearQueryResults = (): void => {
  // Clear query state
}
```

## **3. Component Implementation Plan**

### **A. Query Input Section**
- **Form Layout**: Use `Form` with `TextArea` for query input
- **Advanced Options**: Collapsible section for:
  - `max_results` (number input, default from API)
  - `enable_rerank` (checkbox)
  - `similarity_threshold` (slider, 0-1 range)
- **Submit**: Button with loading state during query

### **B. Results Display Section**
- **Results Overview**: 
  - Total results count
  - Processing time (from metadata)
  - Token usage stats
- **Source Cards**: For each `RAGSource`:
  - Document content preview (truncated)
  - Similarity score (progress bar or badge)
  - File metadata (file_id, chunk_index)
  - Entity/relationship matches if available
- **Empty State**: When no results found
- **Error State**: When query fails

### **C. Layout Structure**
```typescript
<Card title="Query Interface">
  <Space direction="vertical" size="large" className="w-full">
    {/* Query Input Form */}
    <Form>
      <Form.Item name="query">
        <Input.TextArea placeholder="Enter your query..." />
      </Form.Item>
      
      {/* Advanced Options (Collapsible) */}
      <Collapse>
        <Panel header="Advanced Options">
          <Form.Item name="max_results">
            <InputNumber placeholder="Max results" />
          </Form.Item>
          {/* Other options... */}
        </Panel>
      </Collapse>
      
      <Button type="primary" htmlType="submit" loading={querying}>
        Query
      </Button>
    </Form>
    
    {/* Results Section */}
    {queryResults && (
      <Card title="Results">
        {/* Results display... */}
      </Card>
    )}
  </Space>
</Card>
```

## **4. Technical Implementation Details**

### **A. Instance Context**
- Get `ragInstanceId` from URL params via `useParams()`
- Use existing `useRAGInstanceStore` if needed for instance details
- Validate instance exists and is active before allowing queries

### **B. Form Handling**
- Use `Form.useForm()` for form control
- Implement form validation (required query text, valid number ranges)
- Handle form reset after successful query

### **C. State Management**
- Query state in global RAG store for persistence
- Loading states for better UX
- Error handling with proper user feedback

### **D. Results Formatting**
- Format similarity scores as percentages
- Truncate long document content with "Show more" option
- Display metadata in readable format
- Handle empty entity/relationship arrays gracefully

## **5. UI/UX Considerations**

### **A. Responsive Design**
- Mobile-friendly layout
- Proper text wrapping for long content
- Adaptive grid for result cards

### **B. Performance**
- Pagination or virtual scrolling for large result sets
- Debounced search if implementing real-time query
- Proper loading indicators

### **C. Accessibility**
- Proper ARIA labels
- Keyboard navigation support
- Screen reader friendly structure

## **6. Integration Points**

### **A. API Integration**
- Use existing `ApiClient.Rag.queryInstance()` method
- Handle API errors appropriately
- Type safety with generated TypeScript types

### **B. Routing**
- Maintain current URL structure (`/rags/:ragInstanceId/query`)
- No additional routing changes needed

### **C. Permissions**
- Use existing `PermissionGuard` if query requires special permissions
- Check instance ownership/access rights

## **7. Implementation Steps**

1. **Update RAG Store**: Add query state and methods
2. **Create Query Form**: Input section with validation
3. **Implement Results Display**: Card-based results layout
4. **Add Advanced Options**: Collapsible advanced parameters
5. **Handle Loading/Error States**: Proper UX feedback
6. **Style and Polish**: Responsive design and accessibility
7. **Testing**: Manual testing with different query scenarios

## **8. Code Patterns to Follow**

### **A. Component Structure**
```typescript
export const RagQueryTab: React.FC = () => {
  const [form] = Form.useForm<RAGQueryRequest>()
  const { message } = App.useApp()
  const { ragInstanceId } = useParams<{ ragInstanceId: string }>()
  
  // Store usage
  const { queryResults, querying, queryError } = Stores.Rag
  
  // Form submission
  const handleSubmit = async (values: RAGQueryRequest) => {
    try {
      await queryRAGInstance(ragInstanceId!, values)
      message.success('Query completed successfully')
    } catch (error) {
      message.error('Query failed')
    }
  }
  
  // Component JSX...
}
```

### **B. Error Handling**
```typescript
useEffect(() => {
  if (queryError) {
    message.error(queryError)
    clearQueryError() // Clear error after showing
  }
}, [queryError, message])
```

## **9. Generated API Types Available**

From `src/types/api.ts`, we have these interfaces available:

```typescript
// Request interface
interface RAGQueryRequest {
  query: string
  max_results?: number
  enable_rerank?: boolean
  similarity_threshold?: number
}

// Response interface
interface RAGQueryResponse {
  results: RAGSource[]
  token_usage: RAGTokenUsage
  metadata: RAGQueryMetadata
}

// Source information
interface RAGSource {
  document: SimpleVectorDocument
  similarity_score: number
  entity_matches: string[]
  relationship_matches: string[]
}

// Document structure
interface SimpleVectorDocument {
  id: string
  rag_instance_id: string
  file_id: string
  chunk_index: number
  content: string
  content_hash: string
  token_count: number
  metadata: any
  created_at: string
  updated_at: string
}

// Token usage info
interface RAGTokenUsage {
  total_tokens: number
  embedding_tokens: number
  max_total_tokens: number
}

// Query metadata
interface RAGQueryMetadata {
  processing_time_ms: number
  chunks_retrieved: number
  chunks_filtered: number
  rerank_applied: boolean
}
```

## **10. Backend Integration**

The backend API endpoint is already implemented:
- **Endpoint**: `POST /api/rag/instances/{instance_id}/query`
- **Handler**: `query_rag_instance_handler` in `src-tauri/src/api/rag/instances.rs`
- **Authentication**: Required (uses `AuthenticatedUser` middleware)
- **Authorization**: Checks user has access to the RAG instance

## **11. Store Proxy Pattern**

Following the established pattern where components access stores via `Stores.RagName`:

```typescript
// In component
const { queryResults, querying, queryError } = Stores.Rag

// Store methods called directly
await queryRAGInstance(instanceId, queryData)
```

This plan follows all existing patterns in the codebase and provides a comprehensive, user-friendly RAG query interface that integrates seamlessly with the current architecture.