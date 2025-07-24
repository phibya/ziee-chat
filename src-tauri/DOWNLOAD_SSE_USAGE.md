# Download Progress SSE Endpoint Usage

## Endpoint
`GET /api/downloads/subscribe`

## Description
This Server-Sent Events (SSE) endpoint allows clients to subscribe to real-time updates of all active download progress. The endpoint automatically closes the connection when no downloads are active.

## Authentication
Requires authentication via JWT token in the Authorization header.

## Behavior
- **For regular users**: Returns only their own active downloads
- **For admin users**: Returns all active downloads across all users
- **Polling interval**: Updates are sent every 2 seconds
- **Auto-close**: Connection automatically closes when no active downloads remain

## Event Types

### 1. `update` event
Sent when there are active downloads to report.

```json
{
  "type": "update",
  "downloads": [
    {
      "id": "download-uuid",
      "user_id": "user-uuid",
      "provider_id": "provider-uuid",
      "repository_id": "repository-uuid",
      "status": "downloading",
      "progress_data": {
        "current_bytes": 1024000,
        "total_bytes": 10240000,
        "current_file": "model.gguf",
        "total_files": 5,
        "files_completed": 2,
        "download_speed": 1048576,
        "eta_seconds": 10
      },
      "started_at": "2024-01-01T00:00:00Z",
      "created_at": "2024-01-01T00:00:00Z",
      "updated_at": "2024-01-01T00:00:01Z"
    }
  ]
}
```

### 2. `complete` event
Sent when all downloads are completed or when there are no active downloads.

```json
{
  "type": "complete",
  "message": "All downloads completed"
}
```

### 3. `error` event
Sent when an error occurs while fetching download status.

```json
{
  "type": "error",
  "error": "Failed to get downloads: Database error"
}
```

## Client Example (JavaScript/TypeScript)

```typescript
const eventSource = new EventSource('/api/downloads/subscribe', {
  headers: {
    'Authorization': `Bearer ${authToken}`
  }
});

eventSource.addEventListener('update', (event) => {
  const data = JSON.parse(event.data);
  console.log('Active downloads:', data.downloads);
  
  // Update UI with download progress
  data.downloads.forEach(download => {
    updateDownloadProgress(download);
  });
});

eventSource.addEventListener('complete', (event) => {
  const data = JSON.parse(event.data);
  console.log('Downloads complete:', data.message);
  
  // Close the connection
  eventSource.close();
  
  // Update UI to show completion
  showDownloadsComplete();
});

eventSource.addEventListener('error', (event) => {
  const data = JSON.parse(event.data);
  console.error('Download subscription error:', data.error);
  
  // Handle error in UI
  showError(data.error);
});

eventSource.onerror = (error) => {
  console.error('EventSource error:', error);
  eventSource.close();
};
```

## React Hook Example

```typescript
import { useEffect, useState } from 'react';
import { DownloadInstance } from '@/types';

export function useDownloadProgress() {
  const [downloads, setDownloads] = useState<DownloadInstance[]>([]);
  const [isComplete, setIsComplete] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const eventSource = new EventSource('/api/downloads/subscribe', {
      headers: {
        'Authorization': `Bearer ${getAuthToken()}`
      }
    });

    eventSource.addEventListener('update', (event) => {
      const data = JSON.parse(event.data);
      setDownloads(data.downloads);
      setIsComplete(false);
    });

    eventSource.addEventListener('complete', (event) => {
      const data = JSON.parse(event.data);
      setDownloads([]);
      setIsComplete(true);
      eventSource.close();
    });

    eventSource.addEventListener('error', (event) => {
      const data = JSON.parse(event.data);
      setError(data.error);
      eventSource.close();
    });

    eventSource.onerror = () => {
      setError('Connection lost');
      eventSource.close();
    };

    return () => {
      eventSource.close();
    };
  }, []);

  return { downloads, isComplete, error };
}
```

## Notes
- The endpoint automatically handles client disconnects
- Downloads are filtered based on user permissions (admin sees all, users see only their own)
- The connection uses standard SSE keep-alive to maintain the connection
- Progress updates include detailed information about download speed and estimated time remaining