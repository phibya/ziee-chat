import { Button, Card, Tag, Typography, Progress, Spin, App } from 'antd'
import React, { useEffect } from 'react'
import { useParams } from 'react-router-dom'
import {
  subscribeToRAGStatus,
  disconnectRAGStatus,
  Stores,
} from '../../../store'

const { Text } = Typography

export const RagInstanceStatus: React.FC = () => {
  const { ragInstanceId } = useParams<{ ragInstanceId: string }>()
  const { message } = App.useApp()

  // RAG status store
  const { currentStatus, statusLoading, sseConnected, sseError } =
    Stores.RAGStatus

  // Initialize status monitoring
  useEffect(() => {
    if (ragInstanceId) {
      subscribeToRAGStatus(ragInstanceId).catch(console.error)
    }

    return () => {
      disconnectRAGStatus()
    }
  }, [ragInstanceId])

  // Handle status errors
  useEffect(() => {
    if (sseError) {
      message.error(`Status monitoring error: ${sseError}`)
    }
  }, [sseError, message])

  // Don't render if no ragInstanceId
  if (!ragInstanceId) {
    return null
  }

  // Status rendering functions
  const renderConnectionStatus = () => (
    <Card style={{ display: sseConnected ? 'none' : 'block' }}>
      <div className="flex justify-between items-center">
        <div className="flex items-center gap-3">
          <Text strong>Status Monitoring:</Text>
          <Tag color={sseConnected ? 'green' : 'red'}>
            {sseConnected ? 'Connected' : 'Disconnected'}
          </Tag>
          {statusLoading && <Spin size="small" />}
        </div>
        {!sseConnected && !statusLoading && (
          <Button
            size="small"
            type="primary"
            onClick={() => {
              if (ragInstanceId) {
                subscribeToRAGStatus(ragInstanceId).catch(console.error)
              }
            }}
          >
            Connect
          </Button>
        )}
      </div>
      {currentStatus?.updated_at && sseConnected && (
        <Text type="secondary" style={{ fontSize: '11px' }}>
          Last update: {new Date(currentStatus.updated_at).toLocaleTimeString()}
        </Text>
      )}
    </Card>
  )

  const renderInstanceSummary = () => {
    if (!currentStatus) return null

    const completionRate =
      currentStatus.total_files > 0
        ? (currentStatus.processed_files / currentStatus.total_files) * 100
        : 0

    const hasActiveProcessing = currentStatus.processing_files > 0
    const hasErrors = currentStatus.failed_files > 0

    return (
      <Card title="Status">
        <div className="space-y-3">
          {/* Error code display */}
          {currentStatus.error_code !== 'none' && (
            <div className="flex items-center justify-between">
              <Text type="secondary">Error:</Text>
              <Tag color="red">{currentStatus.error_code}</Tag>
            </div>
          )}

          {/* Progress overview */}
          <div className="space-y-2">
            <div className="flex justify-between text-sm">
              <Text type="secondary">Progress:</Text>
              <Text type="secondary">
                {completionRate.toFixed(1)}% complete
              </Text>
            </div>
            <Progress
              percent={completionRate}
              status={
                hasErrors
                  ? 'exception'
                  : hasActiveProcessing
                    ? 'active'
                    : 'success'
              }
              size="small"
            />
          </div>

          {/* Quick stats */}
          <div className="flex justify-between text-xs pt-2">
            <span>
              <Text type="secondary">Total:</Text>
              <Text className="ml-1">{currentStatus.total_files}</Text>
            </span>
            <span>
              <Text type="secondary">Done:</Text>
              <Text className="ml-1">{currentStatus.processed_files}</Text>
            </span>
            <span>
              <Text type="secondary">Active:</Text>
              <Text className="ml-1">{currentStatus.processing_files}</Text>
            </span>
            <span>
              <Text type="secondary">Failed:</Text>
              <Text className="ml-1" type={hasErrors ? 'danger' : 'secondary'}>
                {currentStatus.failed_files}
              </Text>
            </span>
          </div>
        </div>
      </Card>
    )
  }

  const renderCurrentFilesProcessing = () => {
    if (!currentStatus?.current_files_processing?.length) return null

    return (
      <Card title="Currently Processing Files">
        <div className="space-y-2 max-h-40 overflow-y-auto">
          {currentStatus.current_files_processing.map(file => (
            <div
              key={file.file_id}
              className="flex justify-between items-center p-2 rounded"
            >
              <div className="flex-1 min-w-0">
                <Text strong className="truncate block">
                  {file.filename}
                </Text>
                {file.stage && (
                  <Text type="secondary" style={{ fontSize: '11px' }}>
                    Stage: {file.stage}
                  </Text>
                )}
              </div>
              <div className="flex items-center gap-2">
                <Tag
                  color={
                    file.status === 'completed'
                      ? 'green'
                      : file.status === 'failed'
                        ? 'red'
                        : file.status === 'processing'
                          ? 'blue'
                          : 'default'
                  }
                >
                  {file.status}
                </Tag>
                {file.started_at && (
                  <Text type="secondary" style={{ fontSize: '10px' }}>
                    {new Date(file.started_at).toLocaleTimeString()}
                  </Text>
                )}
              </div>
            </div>
          ))}
        </div>
      </Card>
    )
  }

  return (
    <div className="space-y-3">
      {renderConnectionStatus()}
      {currentStatus && (
        <div className={'w-full flex flex-col gap-3'}>
          {renderInstanceSummary()}
          {renderCurrentFilesProcessing()}
        </div>
      )}
    </div>
  )
}
