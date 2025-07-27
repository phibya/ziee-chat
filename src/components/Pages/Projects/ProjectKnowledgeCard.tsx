import { UploadOutlined } from '@ant-design/icons'
import { App, Button, Card, Flex, Progress, Typography, Upload } from 'antd'
import React, { useEffect } from 'react'
import { useParams } from 'react-router-dom'
import { loadProjectFiles, Stores, uploadFilesToProject } from '../../../store'
import { FileCard } from './FileCard'

const { Text } = Typography

export const ProjectKnowledgeCard: React.FC = () => {
  const { message } = App.useApp()
  const { projectId } = useParams<{ projectId: string }>()

  // Projects store
  const { currentProject } = Stores.Projects

  // Project files store
  const { uploading, uploadProgress, showProgress } = Stores.ProjectFiles

  // Get files for this project
  const projectFiles = projectId
    ? Stores.ProjectFiles.filesByProject[projectId] || []
    : []

  useEffect(() => {
    if (projectId) {
      // Load project files
      loadProjectFiles(projectId).catch((error: any) => {
        console.error('Failed to load project files:', error)
      })
    }
  }, [projectId])

  const handleFileUpload = async (files: globalThis.File[]) => {
    if (!currentProject || !projectId) return

    try {
      await uploadFilesToProject(projectId, files)
      message.success(`${files.length} file(s) uploaded successfully`)
    } catch (error) {
      console.error('Failed to upload files:', error)
      message.error('Failed to upload files')
    }
  }

  return (
    <Card
      title="Project knowledge"
      className="w-96 !mt-1"
      classNames={{
        body: 'h-full flex flex-col relative',
      }}
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
        className="!p-0 !m-0
        [&_.ant-upload-drag]:!border-none [&_.ant-upload-drag]:!bg-transparent
        [&_.ant-upload-drag-hover]:!border-dashed
        absolute left-2 right-2 top-2 bottom-12
        "
        openFileDialogOnClick={false}
      />
      {/* Project Description */}
      <Text type="secondary">
        {currentProject?.description ||
          '"Goal: - To completely write the Proposal.tex. There are two...'}
        <Button type="link" size="small" style={{ pointerEvents: 'auto' }}>
          Edit
        </Button>
      </Text>

      {/* Upload Progress */}
      {showProgress && (
        <div className={'py-0 flex flex-col gap-4'}>
          <Text strong>Uploading files...</Text>
          <div className={'py-4 flex flex-col gap-2'}>
            {uploadProgress.map((progress, index) => (
              <div key={index}>
                <Text style={{ fontSize: '12px' }}>{progress.filename}</Text>
                <Progress
                  percent={progress.progress}
                  size="small"
                  status={progress.status === 'error' ? 'exception' : 'active'}
                />
                {progress.error && (
                  <Text type="danger" style={{ fontSize: '10px' }}>
                    {progress.error}
                  </Text>
                )}
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Documents */}
      <Flex justify="space-between" align="center">
        <Typography.Title level={5}>Documents</Typography.Title>
        <Button
          icon={<UploadOutlined />}
          loading={uploading}
          style={{ pointerEvents: 'auto' }}
        >
          + Add Files
        </Button>
      </Flex>

      <div
        style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(3, 1fr)',
          gap: '8px',
          marginTop: 12,
        }}
      >
        {projectFiles.map(file => (
          <FileCard key={file.id} file={file} />
        ))}
      </div>
    </Card>
  )
}
