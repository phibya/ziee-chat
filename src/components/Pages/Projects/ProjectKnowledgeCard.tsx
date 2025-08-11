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
} from 'antd'
import React, { useEffect, useRef, useState } from 'react'
import { createPortal } from 'react-dom'
import { useParams } from 'react-router-dom'
import { useProjectStore } from '../../../store'
import { FileCard } from '../../Common/FileCard.tsx'
import { ProjectInstructionDrawer } from './ProjectInstructionDrawer.tsx'
import { useUpdate } from 'react-use'

const { Text } = Typography

interface ProjectKnowledgeCardProps {
  getHeaderRef?: () => HTMLDivElement | null
}

export const ProjectKnowledgeCard: React.FC<ProjectKnowledgeCardProps> = ({
  getHeaderRef,
}) => {
  const { message } = App.useApp()
  const { projectId } = useParams<{ projectId: string }>()
  const { token } = theme.useToken()
  const [instructionDrawerOpen, setInstructionDrawerOpen] = useState(false)
  const [savingInstruction, setSavingInstruction] = useState(false)
  const fileInputRef = useRef<HTMLInputElement>(null)
  const update = useUpdate()

  // Project store
  const {
    project,
    updateProject,
    uploading,
    uploadProgress,
    files,
    uploadFiles,
    removeUploadProgress,
  } = useProjectStore(projectId)

  useEffect(() => {
    if (getHeaderRef) update()
  }, [])

  // Get files for this project
  const projectFiles = projectId
    ? files.filter(file => file.project_id === projectId)
    : []

  const handleFileUpload = async (files: globalThis.File[]) => {
    if (!project || !projectId) return

    try {
      await uploadFiles(files)
      message.success(`${files.length} file(s) uploaded successfully`)
    } catch (error) {
      console.error('Failed to upload files:', error)
      message.error('Failed to upload files')
    }
  }

  const handleSaveInstruction = async (instruction: string) => {
    if (!project) return

    setSavingInstruction(true)
    try {
      await updateProject({ instruction })
      message.success('Project instructions updated successfully')
    } catch (error) {
      console.error('Failed to update instruction:', error)
      message.error('Failed to update project instructions')
      throw error
    } finally {
      setSavingInstruction(false)
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

  const header = (
    <div className={'w-full flex justify-between'}>
      <Typography.Title level={5} className={'!m-0 !pt-[2px]'}>
        Project knowledge
      </Typography.Title>
      <Button
        icon={<PlusOutlined />}
        onClick={handleAddFilesClick}
        style={{ pointerEvents: 'auto' }}
        loading={uploading}
      />
    </div>
  )

  return (
    <div
      className={'h-full flex flex-col overflow-y-hidden overflow-x-visible'}
    >
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
      {
        //portal if getHeaderRef is provided
        getHeaderRef && getHeaderRef() ? (
          createPortal(header, getHeaderRef()!)
        ) : (
          <div className={'w-full px-3 pt-3 pb-2'}>{header}</div>
        )
      }

      {/* Project Instructions */}
      <div className="flex gap-1 flex-col !px-3">
        <Text strong>Project Instructions</Text>
        <Card
          classNames={{
            body: '!p-2 !px-3 flex items-center justify-between w-full flex gap-2 overflow-hidden',
          }}
        >
          <div className={'flex-1 overflow-hidden'}>
            <Text type="secondary" ellipsis={true}>
              {project?.instruction || 'No instructions provided'}
            </Text>
          </div>
          <div>
            <Button
              type="link"
              size="small"
              style={{ pointerEvents: 'auto' }}
              onClick={() => setInstructionDrawerOpen(true)}
            >
              Edit
            </Button>
          </div>
        </Card>
      </div>

      {/* Upload Progress */}
      {uploadProgress.length > 0 && (
        <div className={'py-0 flex flex-col gap-1 pt-2 px-3'}>
          <Text strong>Uploading files...</Text>
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
      <div className={'!mt-3 flex px-3'}>
        <Text strong>Documents</Text>
      </div>

      <div className={'overflow-y-auto mt-3 flex-1 px-3'}>
        <div>
          <div className="flex gap-2 flex-wrap">
            {/* Show uploading files */}
            {uploading &&
              uploadProgress.map((progress: any, index: number) => (
                <div className={'flex-1 min-w-20 max-w-28'}>
                  <FileCard
                    key={`uploading-${index}`}
                    uploadingFile={progress}
                  />
                </div>
              ))}

            {/* Show existing files */}
            {projectFiles.map((file: any) => (
              <div className={'flex-1 min-w-20 max-w-28'}>
                <FileCard key={file.id} file={file} />
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

      <ProjectInstructionDrawer
        open={instructionDrawerOpen}
        onClose={() => setInstructionDrawerOpen(false)}
        onSave={handleSaveInstruction}
        currentInstruction={project?.instruction}
        loading={savingInstruction}
      />
    </div>
  )
}
