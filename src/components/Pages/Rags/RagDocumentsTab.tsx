import {
  CloseCircleOutlined,
  PlusOutlined,
  UploadOutlined,
  SearchOutlined,
  DeleteOutlined,
} from '@ant-design/icons'
import {
  App,
  Button,
  Card,
  Flex,
  Progress,
  theme,
  Typography,
  Upload,
  List,
  Input,
  Pagination,
  Checkbox,
  Badge,
  Select,
} from 'antd'
import React, { useEffect, useRef, useState } from 'react'
import { useParams } from 'react-router-dom'
import {
  useRAGInstanceStore,
  toggleFileSelection,
  selectAllVisibleFiles,
  clearFileSelection,
  bulkDeleteFiles,
  searchFiles,
  changePage,
  changePageSize,
} from '../../../store/ragInstance'
import { Permission } from '../../../types'
import { PermissionGuard } from '../../Auth/PermissionGuard.tsx'
import { debounce } from '../../../utils/debounce.ts'
import { DivScrollY } from '../../common/DivScrollY.tsx'

const { Text } = Typography

export const RagDocumentsTab: React.FC = () => {
  const { message, modal } = App.useApp()
  const { ragInstanceId } = useParams<{ ragInstanceId: string }>()
  const { token } = theme.useToken()
  const fileInputRef = useRef<HTMLInputElement>(null)

  // Local state for search
  const [localSearchQuery, setLocalSearchQuery] = useState('')

  // RAG instance store
  const {
    ragInstance,
    uploading,
    uploadProgress,
    files,
    uploadFiles,
    removeUploadProgress,
    filesLoading,
    searchQuery,
    currentPage,
    filesPerPage,
    totalFiles,
    selectedFiles,
    bulkOperationInProgress,
  } = useRAGInstanceStore(ragInstanceId)

  // Initialize search state
  useEffect(() => {
    setLocalSearchQuery(searchQuery)
  }, [searchQuery])

  // Handle search input change (only update local state)
  const handleSearchChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value
    setLocalSearchQuery(value)
  }

  // Handle search submit on Enter press
  const handleSearchSubmit = () => {
    if (ragInstanceId) {
      searchFiles(ragInstanceId, localSearchQuery)
    }
  }

  // Handle search clear
  const handleSearchClear = () => {
    if (ragInstanceId) {
      setLocalSearchQuery('')
      searchFiles(ragInstanceId, '')
    }
  }

  // Handle page change
  const handlePageChange = (page: number) => {
    if (ragInstanceId) {
      changePage(ragInstanceId, page)
    }
  }

  // Handle page size change
  const handlePageSizeChange = (pageSize: number) => {
    if (ragInstanceId) {
      changePageSize(ragInstanceId, pageSize)
    }
  }

  // Handle file selection
  const handleFileSelection = (fileId: string, _checked: boolean) => {
    if (ragInstanceId) {
      toggleFileSelection(ragInstanceId, fileId)
    }
  }

  // Handle select all
  const handleSelectAll = (checked: boolean) => {
    if (!ragInstanceId) return

    if (checked) {
      const fileIds = files.map(file => file.file_id)
      selectAllVisibleFiles(ragInstanceId, fileIds)
    } else {
      clearFileSelection(ragInstanceId)
    }
  }

  // Handle bulk delete
  const handleBulkDelete = () => {
    if (!ragInstanceId || selectedFiles.length === 0) return

    modal.confirm({
      title: 'Delete Selected Files',
      content: `Are you sure you want to delete ${selectedFiles.length} selected file(s)? This action cannot be undone.`,
      okText: 'Delete',
      okType: 'danger',
      cancelText: 'Cancel',
      onOk: async () => {
        try {
          await bulkDeleteFiles(ragInstanceId, selectedFiles)
          message.success(
            `${selectedFiles.length} file(s) deleted successfully`,
          )
        } catch (error) {
          console.error('Failed to delete files:', error)
          message.error('Failed to delete files')
        }
      },
    })
  }

  const handleFileUpload = debounce(async (files: globalThis.File[]) => {
    if (!ragInstance || !ragInstanceId) return

    try {
      await uploadFiles(files)
      message.success(`${files.length} file(s) uploaded successfully`)
    } catch (error) {
      console.error('Failed to upload files:', error)
      message.error('Failed to upload files')
    }
  })

  const handleAddFilesClick = () => {
    fileInputRef.current?.click()
  }

  const handleFileInputChange = async (
    event: React.ChangeEvent<HTMLInputElement>,
  ) => {
    const files = event.target.files
    if (files && files.length > 0) {
      await handleFileUpload(Array.from(files))
      // Reset the input so the same files can be selected again if needed
      event.target.value = ''
    }
  }

  const getProcessingStatusText = (status: string) => {
    switch (status) {
      case 'completed':
        return 'Processed'
      case 'processing':
        return 'Processing'
      case 'failed':
        return 'Failed'
      case 'pending':
      default:
        return 'Pending'
    }
  }

  const getProcessingStatusType = (
    status: string,
  ): 'success' | 'processing' | 'error' | 'warning' | 'default' => {
    switch (status) {
      case 'completed':
        return 'success'
      case 'processing':
        return 'processing'
      case 'failed':
        return 'error'
      case 'pending':
      default:
        return 'warning'
    }
  }

  return (
    <Card
      title={
        <div className={'w-full flex justify-between'}>
          <Typography.Title level={5} className={'!m-0 !pt-[2px]'}>
            Documents
          </Typography.Title>
          <PermissionGuard
            permissions={[Permission.RagFilesAdd]}
            type={'disabled'}
          >
            <Button
              icon={<PlusOutlined />}
              onClick={handleAddFilesClick}
              loading={uploading}
              className={'!z-2'}
            />
          </PermissionGuard>
        </div>
      }
      classNames={{
        body: '!px-0 !w-full',
      }}
    >
      <div
        className={'h-full flex flex-col overflow-y-hidden overflow-x-visible'}
      >
        <PermissionGuard permissions={[Permission.RagFilesAdd]}>
          <Upload.Dragger
            multiple
            beforeUpload={(_, fileList) => {
              handleFileUpload(fileList)
              return false
            }}
            showUploadList={false}
            className={`
              opacity-0
              [&_.ant-upload-drag]:!cursor-default
              [&_.ant-upload-drag]:!border-none
              [&_.ant-upload-drag-hover]:!border-dashed
              [&:has(.ant-upload-drag-hover)]:z-[100]
              [&:has(.ant-upload-drag-hover)]:opacity-100
              absolute left-2 right-2 top-3 bottom-2
              transition-opacity duration-300 ease-in-out
              `}
            openFileDialogOnClick={false}
            style={{
              backgroundColor: token.colorBgContainer,
            }}
          >
            <Flex
              className="h-full flex-col items-center justify-center gap-2"
              style={{ pointerEvents: 'none' }}
            >
              <UploadOutlined style={{ fontSize: '24px' }} />
              <Text type="secondary">Drag and drop files here</Text>
            </Flex>
          </Upload.Dragger>
        </PermissionGuard>

        {/* Upload Progress */}
        {uploadProgress.length > 0 && (
          <div className={'py-0 flex flex-col gap-1 pt-2 px-3'}>
            <Text strong>Processing files...</Text>
            <div className={'py-4 flex flex-col gap-2'}>
              {uploadProgress.map((progress: any, index: number) => (
                <div key={index}>
                  <Text style={{ fontSize: '12px' }}>{progress.filename}</Text>
                  <Progress
                    percent={progress.progress}
                    size="small"
                    status={
                      progress.status === 'error' ? 'exception' : 'active'
                    }
                  />
                  {progress.error && (
                    <Flex className={'justify-between w-full'}>
                      <Text type="danger">{progress.error}</Text>
                      <Button
                        type="text"
                        icon={<CloseCircleOutlined />}
                        size="small"
                        onClick={() => {
                          removeUploadProgress(progress.id)
                        }}
                      />
                    </Flex>
                  )}
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Search Input */}
        <div className="mt-0 px-3">
          <Input
            placeholder="Search files..."
            prefix={<SearchOutlined />}
            value={localSearchQuery}
            onChange={handleSearchChange}
            onPressEnter={handleSearchSubmit}
            onClear={handleSearchClear}
            allowClear
          />
        </div>

        {/* File list header - fixed */}
        {files.length > 0 && (
          <div className="mt-3 flex items-center justify-between px-3">
            <div className="flex items-center gap-2">
              {selectedFiles.length > 0 ? (
                <>
                  <Text strong>{selectedFiles.length} selected</Text>
                  <PermissionGuard
                    permissions={[Permission.RagFilesRemove]}
                    type="disabled"
                  >
                    <Button
                      type="text"
                      icon={<DeleteOutlined />}
                      onClick={handleBulkDelete}
                      loading={bulkOperationInProgress}
                      danger
                      size="small"
                    >
                      Delete
                    </Button>
                  </PermissionGuard>
                </>
              ) : (
                totalFiles > filesPerPage && (
                  <Text type="secondary">
                    Showing {Math.min(filesPerPage, files.length)} of{' '}
                    {totalFiles} files
                  </Text>
                )
              )}
            </div>
            <Button
              type="link"
              size="small"
              onClick={() => {
                const allSelected =
                  selectedFiles.length === files.length && files.length > 0
                handleSelectAll(!allSelected)
              }}
            >
              {selectedFiles.length === files.length && files.length > 0
                ? 'Deselect All'
                : 'Select All'}
            </Button>
          </div>
        )}

        {/* Scrollable file list */}
        <DivScrollY className={'mt-1 flex-1 max-h-96'}>
          <List
            className={'!px-3'}
            loading={filesLoading}
            dataSource={files}
            renderItem={file => (
              <List.Item key={file.file_id}>
                <div className="flex items-center w-full gap-2">
                  <Checkbox
                    checked={selectedFiles.includes(file.file_id)}
                    onChange={e =>
                      handleFileSelection(file.file_id, e.target.checked)
                    }
                  />
                  <Text className="flex-1 truncate">{file.filename}</Text>
                  <Badge
                    status={getProcessingStatusType(file.processing_status)}
                    text={getProcessingStatusText(file.processing_status)}
                  />
                </div>
              </List.Item>
            )}
            locale={{
              emptyText: localSearchQuery
                ? 'No files match your search'
                : 'No files uploaded yet',
            }}
          />
        </DivScrollY>

        {/* Fixed pagination at bottom */}
        {totalFiles > 0 && (
          <div className="mt-2 flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between px-3">
            {/* Page size selector */}
            <div className="flex items-center gap-2">
              <Text type="secondary">Show:</Text>
              <Select
                value={filesPerPage}
                onChange={handlePageSizeChange}
                size="small"
                style={{ width: 70 }}
                options={[
                  { label: '5', value: 5 },
                  { label: '10', value: 10 },
                  { label: '20', value: 20 },
                  { label: '50', value: 50 },
                ]}
              />
              <Text type="secondary">files per page</Text>
            </div>

            {/* Pagination controls */}
            <Pagination
              current={currentPage}
              pageSize={filesPerPage}
              total={totalFiles}
              onChange={handlePageChange}
              showSizeChanger={false}
              showQuickJumper={totalFiles > filesPerPage * 10}
              showTotal={(total, range) =>
                `${range[0]}-${range[1]} of ${total} files`
              }
              size="small"
              className={'z-1'}
            />
          </div>
        )}

        {/* Hidden file input */}
        <input
          ref={fileInputRef}
          type="file"
          multiple
          style={{ display: 'none' }}
          onChange={handleFileInputChange}
        />
      </div>
    </Card>
  )
}
