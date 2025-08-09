import {
  CloseOutlined,
  DeleteOutlined,
  DownloadOutlined,
} from '@ant-design/icons'
import { App, Button, Card, Spin, theme, Typography } from 'antd'
import React, { useEffect, useState } from 'react'
import {
  deleteFile,
  getFileContent,
  getFileThumbnail,
  getFileThumbnails,
} from '../../store'
import { Drawer } from './Drawer.tsx'
import { formatFileSize, isTextFile } from '../../utils/fileUtils.ts'
import type { File, FileUploadProgress } from '../../types'
import { generateFileDownloadToken } from '../../store/files.ts'

const { Text } = Typography

interface FileCardProps {
  file?: File
  uploadingFile?: FileUploadProgress
  size?: number
  showFileName?: boolean // Whether to show the file name below the card
  onRemove?: (fileId: string) => void // remove from the list, not delete from server
  canRemove?: boolean // Whether the remove button should be shown
  canDelete?: boolean // Whether the delete button should be shown
  onDelete?: (fileId: string) => void // delete from server
}

interface FileModalContentProps {
  file: File
}

const FileModalContent: React.FC<FileModalContentProps> = ({ file }) => {
  const [thumbnails, setThumbnails] = useState<string[]>([])
  const [thumbnailOrder, setThumbnailOrder] = useState<number[]>([])
  const [loading, setLoading] = useState(true)
  const [downloadToken, setDownloadToken] = useState<string | null>(null)
  const [isAnimating, setIsAnimating] = useState(false)
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
        setThumbnailOrder(
          Array.from({ length: thumbnailUrls.length }, (_, i) => i),
        )
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

  const handleThumbnailClick = () => {
    if (thumbnailOrder.length <= 1 || isAnimating) return

    setIsAnimating(true)

    // Add a slight delay to create a more smooth transition effect
    setTimeout(() => {
      // Move the front thumbnail to the end
      const newOrder = [...thumbnailOrder]
      const frontIndex = newOrder.shift()
      if (frontIndex !== undefined) {
        newOrder.push(frontIndex)
      }
      setThumbnailOrder(newOrder)
    }, 50)

    // Reset animation state after animation completes
    setTimeout(() => {
      setIsAnimating(false)
    }, 350)
  }

  return (
    <div className="flex flex-col items-center gap-4 py-4">
      <div className="text-center">
        {loading ? (
          <div className="text-6xl mb-4">⏳</div>
        ) : thumbnails.length > 0 ? (
          <div className="mb-4 relative">
            {/* Stack multiple thumbnails */}
            <div
              className="relative cursor-pointer group"
              style={{ width: 'fit-content', margin: '0 auto' }}
              onClick={handleThumbnailClick}
              title={
                thumbnailOrder.length > 1
                  ? 'Click to cycle through thumbnails'
                  : ''
              }
            >
              {thumbnailOrder.map((originalIndex, displayIndex) => (
                <img
                  key={`${originalIndex}-${displayIndex}`}
                  src={thumbnails[originalIndex]}
                  alt={`${file.filename} - Page ${originalIndex + 1}`}
                  className="max-w-full max-h-96 object-contain rounded shadow transition-all duration-300 ease-in-out hover:scale-105"
                  style={{
                    position: displayIndex === 0 ? 'relative' : 'absolute',
                    top: displayIndex === 0 ? 0 : `${displayIndex * 8}px`,
                    left: displayIndex === 0 ? 0 : `${displayIndex * 8}px`,
                    zIndex: thumbnailOrder.length - displayIndex,
                    transform: `${displayIndex > 0 ? 'rotate(2deg)' : 'none'} ${
                      isAnimating && displayIndex === 0
                        ? 'scale(0.95) translateY(-5px)'
                        : ''
                    }`,
                    opacity: isAnimating && displayIndex === 0 ? 0.8 : 1,
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

export const FileCard: React.FC<FileCardProps> = ({
  file,
  uploadingFile,
  size = 111,
  showFileName = true,
  canRemove = true,
  canDelete = true,
  onRemove,
  onDelete,
}) => {
  const { message, modal } = App.useApp()
  const { token } = theme.useToken()

  const [isDrawerOpen, setIsDrawerOpen] = useState(false)
  const [fileContent, setFileContent] = useState<string>('')
  const [loading, setLoading] = useState(false)
  const [thumbnailUrl, setThumbnailUrl] = useState<string | null>(null)

  useEffect(() => {
    if (!file || file.thumbnail_count === 0) return
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
  }, [file?.id])

  const handleFileDelete = async (fileId: string) => {
    if (!canDelete) return
    try {
      await deleteFile(fileId)
      message.success('File deleted successfully')
      onDelete?.(fileId)
    } catch (error) {
      console.error('Failed to delete file:', error)
      message.error('Failed to delete file')
    }
  }

  const handleFileRemove = async (fileId: string) => {
    if (!canRemove) return
    onRemove?.(fileId)
  }

  const handleCardClick = async () => {
    if (!file || uploadingFile) return

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

  if (uploadingFile) {
    return (
      <div
        style={{
          width: size,
        }}
      >
        <Card
          size="small"
          className="relative"
          style={{
            height: size,
          }}
          styles={{
            body: {
              height: '100%',
              display: 'flex',
              flexDirection: 'column',
              justifyContent: 'center',
              position: 'relative',
              padding: '8px',
            },
          }}
        >
          <Spin />

          {/* Remove button for uploading files */}
          {onRemove && (
            <Button
              danger
              size="small"
              icon={<CloseOutlined />}
              onClick={() => onRemove(uploadingFile.id)}
              className="!absolute top-1 right-1"
            />
          )}

          {/* File extension */}
          <Text
            className="absolute top-1 left-1 rounded px-1 !text-[9px]"
            style={{
              backgroundColor: token.colorBgContainer,
            }}
            strong
          >
            {uploadingFile.filename.split('.').pop()?.toUpperCase() || 'FILE'}
          </Text>

          {/* Upload status */}
          {uploadingFile.status === 'error' && (
            <Text
              className="absolute top-1 right-1 rounded px-1 !text-[9px]"
              style={{
                backgroundColor: token.colorError,
                color: token.colorWhite,
              }}
            >
              ERROR
            </Text>
          )}
        </Card>
        <div
          className="w-full text-center text-xs text-ellipsis overflow-hidden"
          style={{
            display: showFileName ? 'block' : 'none',
          }}
        >
          <Text className={'whitespace-nowrap'}>{uploadingFile.filename}</Text>
        </div>
      </div>
    )
  }

  if (!file) {
    return null
  }

  return (
    <div
      style={{
        width: size,
      }}
    >
      <Card
        size="small"
        className="group relative cursor-pointer"
        style={{
          height: size,
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
        {/* Delete/Remove button - only visible on hover */}
        {(canDelete || canRemove) && (
          <Button
            danger
            size="small"
            icon={<DeleteOutlined />}
            onClick={e => {
              e.stopPropagation()
              if (canDelete) {
                handleFileDelete(file.id)
              } else {
                handleFileRemove(file.id)
              }
            }}
            style={{
              display: canRemove ? 'block' : 'none',
            }}
            className="!absolute top-1 right-1 opacity-0
                    group-hover:opacity-100 transition-opacity bg-transparent"
          />
        )}

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
      <div
        className="w-full text-center text-xs text-ellipsis overflow-hidden"
        style={{
          display: showFileName ? 'block' : 'none',
        }}
      >
        <Text className={'whitespace-nowrap'}>{file.filename}</Text>
      </div>

      {/* Drawer for text file content */}
      <Drawer
        title={file.filename}
        open={isDrawerOpen}
        onClose={() => setIsDrawerOpen(false)}
        width={600}
        footer={[<Button onClick={() => setIsDrawerOpen(false)}>Close</Button>]}
        classNames={{
          body: '!px-3 !pb-1 !pt-0',
        }}
      >
        <Card className="font-mono text-sm whitespace-pre-wrap p-4 rounded max-h-full overflow-auto">
          {fileContent}
        </Card>
      </Drawer>
    </div>
  )
}
