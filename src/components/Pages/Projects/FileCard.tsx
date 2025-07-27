import { DeleteOutlined, DownloadOutlined } from '@ant-design/icons'
import { App, Button, Card, theme, Typography } from 'antd'
import React, { useEffect, useState } from 'react'
import {
  deleteProjectFile,
  getFileContent,
  getFileThumbnail,
  getFileThumbnails,
  Stores,
} from '../../../store'
import { Drawer } from '../../common/Drawer'
import { formatFileSize, isTextFile } from '../../../utils/fileUtils'
import type { File } from '../../../types'
import { generateFileDownloadToken } from '../../../store/projectFiles.ts'

const { Text } = Typography

interface FileCardProps {
  file: File
}

interface FileModalContentProps {
  file: File
}

const FileModalContent: React.FC<FileModalContentProps> = ({ file }) => {
  const [thumbnails, setThumbnails] = useState<string[]>([])
  const [loading, setLoading] = useState(true)
  const [downloadToken, setDownloadToken] = useState<string | null>(null)
  const { message } = App.useApp()

  useEffect(() => {
    generateFileDownloadToken(file.id)
      .then(({ token }) => {
        setDownloadToken(token)
      })
      .catch(error => {
        console.error('Failed to generate download token:', error)
        message.error('Failed to generate download link')
      })
  }, [file.id])

  useEffect(() => {
    const loadThumbnails = async () => {
      if (file.thumbnail_count === 0) {
        setLoading(false)
        return
      }
      setLoading(true)
      try {
        const thumbnailUrls = await getFileThumbnails(
          file.id,
          file.thumbnail_count || 1,
        )
        setThumbnails(thumbnailUrls)
      } catch (error) {
        console.debug('Failed to load thumbnails:', error)
      } finally {
        setLoading(false)
      }
    }

    loadThumbnails()

    // Cleanup function to revoke object URLs
    return () => {
      thumbnails.forEach(url => {
        window.URL.revokeObjectURL(url)
      })
    }
  }, [file.id, file.thumbnail_count])

  return (
    <div className="flex flex-col items-center gap-4 py-4">
      <div className="text-center">
        {loading ? (
          <div className="text-6xl mb-4">⏳</div>
        ) : thumbnails.length > 0 ? (
          <div className="mb-4 relative">
            {/* Stack multiple thumbnails */}
            <div
              className="relative"
              style={{ width: 'fit-content', margin: '0 auto' }}
            >
              {thumbnails.map((url, index) => (
                <img
                  key={index}
                  src={url}
                  alt={`${file.filename} - Page ${index + 1}`}
                  className="max-w-full max-h-96 object-contain rounded shadow"
                  style={{
                    position: index === 0 ? 'relative' : 'absolute',
                    top: index === 0 ? 0 : `${index * 8}px`,
                    left: index === 0 ? 0 : `${index * 8}px`,
                    zIndex: thumbnails.length - index,
                    transform: index > 0 ? 'rotate(2deg)' : 'none',
                  }}
                />
              ))}
            </div>
          </div>
        ) : (
          <div>
            <div className="text-6xl mb-4">📄</div>
            <Text type={'secondary'}>
              Preview not available for this file type
            </Text>
          </div>
        )}
        <div className="pt-4">
          <Text type={'secondary'}>
            File size: {formatFileSize(file.file_size)}
          </Text>
        </div>
      </div>
      <Button type={'primary'}>
        <div className="flex justify-center">
          <a
            href={`/api/files/${file.id}/download-with-token?token=${downloadToken}`}
            download={file.filename}
            className="ant-btn ant-btn-primary"
          >
            <Typography.Text>
              <DownloadOutlined /> Download File
            </Typography.Text>
          </a>
        </div>
      </Button>
    </div>
  )
}

export const FileCard: React.FC<FileCardProps> = ({ file }) => {
  const { message, modal } = App.useApp()
  const { currentProject } = Stores.Projects
  const { token } = theme.useToken()

  const [isDrawerOpen, setIsDrawerOpen] = useState(false)
  const [fileContent, setFileContent] = useState<string>('')
  const [loading, setLoading] = useState(false)
  const [thumbnailUrl, setThumbnailUrl] = useState<string | null>(null)

  useEffect(() => {
    if (file.thumbnail_count === 0) return
    const loadThumbnail = async () => {
      const url = await getFileThumbnail(file.id)
      setThumbnailUrl(url)
    }

    loadThumbnail().catch(error => {
      console.debug('Failed to load thumbnail:', error)
    })

    // Cleanup function to revoke object URL
    return () => {
      if (thumbnailUrl) {
        window.URL.revokeObjectURL(thumbnailUrl)
      }
    }
  }, [file.id])

  const handleFileDelete = async (fileId: string) => {
    if (!currentProject?.id) return

    try {
      await deleteProjectFile(currentProject.id, fileId)
      message.success('File deleted successfully')
    } catch (error) {
      console.error('Failed to delete file:', error)
      message.error('Failed to delete file')
    }
  }

  const handleCardClick = async () => {
    if (isTextFile(file.filename)) {
      // Open drawer for text files
      setLoading(true)
      try {
        const content = await getFileContent(file.id)
        setFileContent(content)
        setIsDrawerOpen(true)
      } catch (error) {
        console.error('Failed to fetch file content:', error)
        message.error('Failed to load file content')
      } finally {
        setLoading(false)
      }
    } else {
      // Open modal for non-text files
      modal.info({
        icon: null, // No icon for file modal
        title: file.filename,
        width: 600,
        content: <FileModalContent file={file} />,
        footer: null, // Footer is handled within FileModalContent
        closable: true,
        maskClosable: true,
      })
    }
  }

  return (
    <div
      style={{
        width: '111px',
      }}
    >
      <Card
        size="small"
        className="group relative cursor-pointer"
        style={{
          height: '111px',
          backgroundImage: thumbnailUrl ? `url(${thumbnailUrl})` : undefined,
          backgroundSize: 'cover',
          backgroundPosition: 'center',
          backgroundRepeat: 'no-repeat',
        }}
        onClick={handleCardClick}
        loading={loading}
        styles={{
          body: {
            height: '100%',
            display: 'flex',
            flexDirection: 'column',
            justifyContent: 'center',
            position: 'relative',
          },
        }}
      >
        {/* Delete button - only visible on hover */}
        <Button
          danger
          size="small"
          icon={<DeleteOutlined />}
          onClick={e => {
            e.stopPropagation()
            handleFileDelete(file.id).catch(error => {
              console.error('Failed to delete file:', error)
            })
          }}
          className="!absolute top-1 right-1 opacity-0
                    group-hover:opacity-100 transition-opacity bg-transparent"
        />

        <Text
          className="absolute top-1 left-1 rounded px-1 !text-[9px]"
          style={{
            backgroundColor: token.colorBgContainer,
          }}
          strong
        >
          {file.filename.split('.').pop()?.toUpperCase() || 'FILE'}
        </Text>

        <Text
          className="absolute bottom-1 right-1 rounded px-1  !text-[9px]"
          style={{
            backgroundColor: token.colorBgContainer,
          }}
        >
          {formatFileSize(file.file_size)}
        </Text>
      </Card>
      <div className="w-full text-center text-xs text-ellipsis overflow-hidden">
        <Text className={'whitespace-nowrap'}>{file.filename}</Text>
      </div>

      {/* Drawer for text file content */}
      <Drawer
        title={file.filename}
        open={isDrawerOpen}
        onClose={() => setIsDrawerOpen(false)}
        width={600}
        footer={[<Button onClick={() => setIsDrawerOpen(false)}>Close</Button>]}
      >
        <Card className="font-mono text-sm whitespace-pre-wrap p-4 rounded max-h-full overflow-auto">
          {fileContent}
        </Card>
      </Drawer>
    </div>
  )
}
