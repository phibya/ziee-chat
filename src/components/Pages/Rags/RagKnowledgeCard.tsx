import {
  CloseCircleOutlined,
  PlusOutlined,
  UploadOutlined,
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
  Tag,
} from 'antd'
import React, { useEffect, useRef } from 'react'
import { createPortal } from 'react-dom'
import { useParams } from 'react-router-dom'
import { useRAGInstanceStore } from '../../../store'
import { FileCard } from '../../common/FileCard.tsx'
import { useUpdate } from 'react-use'
import { Permission } from '../../../types'
import { PermissionGuard } from '../../Auth/PermissionGuard.tsx'
import { hasPermission } from '../../../permissions/utils.ts'

const { Text } = Typography

interface RagKnowledgeCardProps {
  getHeaderRef?: () => HTMLDivElement | null
}

export const RagKnowledgeCard: React.FC<RagKnowledgeCardProps> = ({
  getHeaderRef,
}) => {
  const { message } = App.useApp()
  const { ragInstanceId } = useParams<{ ragInstanceId: string }>()
  const { token } = theme.useToken()
  const fileInputRef = useRef<HTMLInputElement>(null)
  const update = useUpdate()

  // RAG instance store
  const {
    ragInstance,
    uploading,
    uploadProgress,
    files,
    uploadFiles,
    removeUploadProgress,
  } = useRAGInstanceStore(ragInstanceId)

  useEffect(() => {
    if (getHeaderRef) update()
  }, [])

  // Get files for this RAG instance
  const instanceFiles = ragInstanceId
    ? files.filter(file => file.rag_instance_id === ragInstanceId)
    : []

  const handleFileUpload = async (files: globalThis.File[]) => {
    if (!ragInstance || !ragInstanceId) return

    try {
      await uploadFiles(files)
      message.success(`${files.length} file(s) uploaded successfully`)
    } catch (error) {
      console.error('Failed to upload files:', error)
      message.error('Failed to upload files')
    }
  }

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

  const getProcessingStatusColor = (status: string) => {
    switch (status) {
      case 'completed':
        return 'green'
      case 'processing':
        return 'blue'
      case 'failed':
        return 'red'
      case 'pending':
      default:
        return 'orange'
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

  const header = (
    <div className={'w-full flex justify-between'}>
      <Typography.Title level={5} className={'!m-0 !pt-[2px]'}>
        RAG Knowledge
      </Typography.Title>
      <PermissionGuard
        permissions={[Permission.RagFilesAdd]}
        type={'disabled'}
      >
        <Button
          icon={<PlusOutlined />}
          onClick={handleAddFilesClick}
          style={{ pointerEvents: 'auto' }}
          loading={uploading}
        />
      </PermissionGuard>
    </div>
  )

  return (
    <div
      className={'h-full flex flex-col overflow-y-hidden overflow-x-visible'}
    >
      <PermissionGuard permissions={[Permission.RagFilesAdd]}>
        <Upload.Dragger
          multiple
          beforeUpload={(_, fileList) => {
            handleFileUpload(fileList).catch(error => {
              console.error('Failed to upload files:', error)
            })
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
      {
        //portal if getHeaderRef is provided
        getHeaderRef && getHeaderRef() ? (
          createPortal(header, getHeaderRef()!)
        ) : (
          <div className={'w-full px-3 pt-3 pb-2'}>{header}</div>
        )
      }

      {/* RAG Instance Info */}
      <div className="flex gap-1 flex-col !px-3">
        <Text strong>Instance Information</Text>
        <Card
          classNames={{
            body: '!p-2 !px-3 flex flex-col gap-2',
          }}
        >
          <div className={'flex items-center justify-between'}>
            <Text type="secondary">Engine Type:</Text>
            <Tag color="blue">
              {ragInstance?.engine_type === 'simple_vector' ? 'Vector' : 'Graph'}
            </Tag>
          </div>
          <div className={'flex items-center justify-between'}>
            <Text type="secondary">Status:</Text>
            <Tag color={ragInstance?.is_active ? 'green' : 'red'}>
              {ragInstance?.is_active ? 'Active' : 'Inactive'}
            </Tag>
          </div>
        </Card>
      </div>

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
                  status={progress.status === 'error' ? 'exception' : 'active'}
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

      {/* Documents */}
      <div className={'!mt-3 flex px-3 justify-between'}>
        <Text strong>Documents</Text>
        <Text type="secondary" style={{ fontSize: '12px' }}>
          {instanceFiles.length} files
        </Text>
      </div>

      <div className={'overflow-y-auto mt-3 flex-1 px-3'}>
        <div>
          <div className="flex gap-2 flex-wrap">
            {/* Show uploading files */}
            {uploading &&
              uploadProgress.map((progress: any, index: number) => (
                <div key={`uploading-${index}`} className={'flex-1 min-w-20 max-w-28'}>
                  <FileCard
                    uploadingFile={progress}
                  />
                </div>
              ))}

            {/* Show existing files with processing status */}
            {instanceFiles.map((file: any) => (
              <div key={file.id} className={'flex-1 min-w-20 max-w-28'}>
                <div className="relative">
                  <FileCard
                    file={file}
                    canDelete={hasPermission([Permission.RagFilesRemove])}
                    canRemove={hasPermission([Permission.RagFilesRemove])}
                  />
                  {/* Processing Status Badge */}
                  <div className="absolute top-1 right-1">
                    <Tag 
                      color={getProcessingStatusColor(file.processing_status)}
                      style={{ fontSize: '10px', lineHeight: '14px' }}
                    >
                      {getProcessingStatusText(file.processing_status)}
                    </Tag>
                  </div>
                </div>
              </div>
            ))}
            {new Array(10).fill(0).map((_, i) => (
              <div key={i} className={'flex-1 min-w-20 max-w-28'} />
            ))}
          </div>
        </div>
      </div>

      {/* Hidden file input */}
      <input
        ref={fileInputRef}
        type="file"
        multiple
        style={{ display: 'none' }}
        onChange={handleFileInputChange}
      />
    </div>
  )
}