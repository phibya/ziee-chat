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
import React, { useRef, useState } from 'react'
import { useParams } from 'react-router-dom'
import { useProjectStore } from '../../../store'
import { FileCard } from '../../Common/FileCard.tsx'
import { ProjectInstructionDrawer } from './ProjectInstructionDrawer.tsx'

const { Text } = Typography

export const ProjectKnowledgeCard: React.FC = () => {
  const { message } = App.useApp()
  const { projectId } = useParams<{ projectId: string }>()
  const { token } = theme.useToken()
  const [instructionDrawerOpen, setInstructionDrawerOpen] = useState(false)
  const [savingInstruction, setSavingInstruction] = useState(false)
  const fileInputRef = useRef<HTMLInputElement>(null)

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

  return (
    <div>
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
      <div className={'w-full flex justify-between'}>
        <Typography.Title level={5} className={'pb-2'}>
          Project knowledge
        </Typography.Title>
        <Button
          icon={<PlusOutlined />}
          onClick={handleAddFilesClick}
          style={{ pointerEvents: 'auto' }}
          loading={uploading}
        />
      </div>
      {/* Project Instructions */}
      <Flex className="gap-1 flex-col">
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
      </Flex>

      {/* Upload Progress */}
      {uploadProgress.length > 0 && (
        <div className={'py-0 flex flex-col gap-1 pt-2'}>
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
      <Flex justify="space-between" align="center" className={'!mt-3'}>
        <Text strong>Documents</Text>
      </Flex>

      <div className={'overflow-y-auto mt-3 flex-1'}>
        <div className="flex gap-2 flex-wrap">
          {/* Show uploading files */}
          {uploading &&
            uploadProgress.map((progress: any, index: number) => (
              <FileCard
                key={`uploading-${index}`}
                uploadingFile={progress}
                size={90}
              />
            ))}

          {/* Show existing files */}
          {projectFiles.map((file: any) => (
            <FileCard key={file.id} file={file} size={103} />
          ))}
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
