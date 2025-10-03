import { Flex, theme, Typography, Upload } from 'antd'
import { BsFileEarmarkPlus } from 'react-icons/bs'
import { Permission } from '../../../../types'
import { PermissionGuard } from '../../../Auth/PermissionGuard'

const { Text } = Typography

interface FileUploadAreaProps {
  onFileUpload: (files: File[]) => Promise<void> | void
}

export const FileUploadArea = ({ onFileUpload }: FileUploadAreaProps) => {
  const { token } = theme.useToken()

  return (
    <PermissionGuard permissions={[Permission.ChatCreate]}>
      <Upload.Dragger
        multiple
        beforeUpload={(_, fileList) => {
          if (fileList) {
            onFileUpload(fileList)?.catch?.(error => {
              console.error('Failed to upload files:', error)
            })
          }
          return false
        }}
        showUploadList={false}
        className={`
          opacity-0
          [&_.ant-upload-drag]:!cursor-default
          [&_.ant-upload-drag]:!border-none
          [&_.ant-upload-drag-hover]:!border-dashed
          [&:has(.ant-upload-drag-hover)]:opacity-100
          [&:has(.ant-upload-drag-hover)]:!z-500
          absolute inset-0
          transition-opacity duration-300 ease-in-out
        `}
        openFileDialogOnClick={false}
        style={{
          backgroundColor: token.colorBgLayout,
          borderRadius: token.borderRadius,
        }}
      >
        <Flex
          className="h-full flex-col items-center justify-center gap-3"
          style={{ pointerEvents: 'none' }}
        >
          <BsFileEarmarkPlus className={'text-2xl'} />
          <Text type="secondary">Drop files here to upload</Text>
        </Flex>
      </Upload.Dragger>
    </PermissionGuard>
  )
}
